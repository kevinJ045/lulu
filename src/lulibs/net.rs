use std::sync::{Arc};
use hyper::body::{Bytes};
use http_body_util::BodyExt;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::{TokioIo};
use http_body_util::Full;
use hyper::http::{HeaderName, HeaderValue};
use hyper::HeaderMap;
use reqwest::{Client, Method};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::Mutex as TokioMutex;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::{WebSocketStream, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use crate::lulibs::bytes::LuluByteArray;
use crate::lulibs::threads::TOK_ASYNC_HANDLES;
use crate::ops::std::create_std_module;
use std::collections::HashMap;
use mlua::Error as LuaError;
use std::net::SocketAddr;
use std::convert::Infallible;

#[derive(Clone)]
pub struct LuluTcpStream {
  reader: Arc<TokioMutex<ReadHalf<TcpStream>>>,
  writer: Arc<TokioMutex<WriteHalf<TcpStream>>>,
}

impl LuluTcpStream {
  pub fn new(stream: TcpStream) -> Self {
    let (reader, writer) = tokio::io::split(stream);
    Self {
      reader: Arc::new(TokioMutex::new(reader)),
      writer: Arc::new(TokioMutex::new(writer)),
    }
  }
}

impl mlua::UserData for LuluTcpStream {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_async_method("read", |_, this, n: Option<usize>| async move {
      let mut reader = this.reader.lock().await;
      let n = n.unwrap_or(1024);
      let mut buf = vec![0; n];
      match reader.read(&mut buf).await {
        Ok(0) => Ok(None), // EOF
        Ok(bytes_read) => {
          buf.truncate(bytes_read);
          Ok(Some(LuluByteArray { bytes: buf }))
        }
        Err(e) => Err(mlua::Error::external(e)),
      }
    });

    methods.add_async_method("write", |_, this, data: mlua::Value| async move {
      let mut writer = this.writer.lock().await;
      let bytes = match data {
        mlua::Value::String(s) => s.as_bytes().to_vec(),
        mlua::Value::UserData(ud) => ud.borrow::<LuluByteArray>()?.bytes.clone(),
        _ => return Err(mlua::Error::external("string or ByteArray")),
      };
      writer
        .write_all(&bytes)
        .await
        .map_err(mlua::Error::external)?;
      Ok(())
    });

    methods.add_async_method("close", |_, this, ()| async move {
      let mut writer = this.writer.lock().await;
      writer.shutdown().await.map_err(mlua::Error::external)?;
      Ok(())
    });
  }
}

#[derive(Clone)]
pub struct LuluTcpListener {
  pub listener: Arc<TcpListener>,
}

impl mlua::UserData for LuluTcpListener {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_async_method("accept", |_, this, ()| async move {
      let (socket, _) = this
        .listener
        .accept()
        .await
        .map_err(mlua::Error::external)?;
      Ok(LuluTcpStream::new(socket))
    });
  }
}

#[derive(Clone)]
pub struct LuluUdpSocket {
  socket: Arc<UdpSocket>,
}

impl LuluUdpSocket {
  pub fn new(socket: UdpSocket) -> Self {
    Self {
      socket: Arc::new(socket),
    }
  }
}

impl mlua::UserData for LuluUdpSocket {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_async_method(
      "send_to",
      |_, this, (addr, data): (String, mlua::Value)| async move {
        let bytes = match data {
          mlua::Value::String(s) => s.as_bytes().to_vec(),
          mlua::Value::UserData(ud) => ud.borrow::<LuluByteArray>()?.bytes.clone(),
          _ => return Err(mlua::Error::external("string or ByteArray")),
        };
        let sent = this
          .socket
          .send_to(&bytes, &addr)
          .await
          .map_err(mlua::Error::external)?;
        Ok(sent)
      },
    );

    methods.add_async_method("recv_from", |lua, this, n: Option<usize>| async move {
      let n = n.unwrap_or(65535);
      let mut buf = vec![0; n];
      let (len, addr) = this
        .socket
        .recv_from(&mut buf)
        .await
        .map_err(mlua::Error::external)?;
      buf.truncate(len);
      let result = lua.create_table()?;
      result.set("data", LuluByteArray { bytes: buf })?;
      result.set("addr", addr.to_string())?;
      Ok(result)
    });
  }
}

#[derive(Clone)]
pub struct LuluWebSocket {
  stream:
    Arc<TokioMutex<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>,
}

impl LuluWebSocket {
  pub fn new(
    stream: WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
  ) -> Self {
    Self {
      stream: Arc::new(TokioMutex::new(stream)),
    }
  }
}

impl mlua::UserData for LuluWebSocket {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_async_method("read", |lua, this, ()| async move {
      let mut stream = this.stream.lock().await;
      match stream.next().await {
        Some(Ok(msg)) => {
          match msg {
            Message::Text(t) => Ok(mlua::Value::String(lua.create_string(&t)?)),
            Message::Binary(b) => Ok(mlua::Value::UserData(
              lua.create_userdata(LuluByteArray { bytes: b.to_vec() })?,
            )),
            _ => Ok(mlua::Value::Nil), // Ignore Ping/Pong/Frame/Close
          }
        }
        Some(Err(e)) => Err(mlua::Error::external(e)),
        _ => Ok(mlua::Value::Nil), // Stream closed
      }
    });

    methods.add_async_method("write", |_, this, data: mlua::Value| async move {
      let mut stream = this.stream.lock().await;
      let msg = match data {
        mlua::Value::String(s) => Message::Text(s.to_str()?.to_string().into()),
        mlua::Value::UserData(ud) => {
          Message::Binary(ud.borrow::<LuluByteArray>()?.bytes.clone().into())
        }
        _ => return Err(mlua::Error::external("string or ByteArray")),
      };
      stream.send(msg).await.map_err(mlua::Error::external)?;
      Ok(())
    });

    methods.add_async_method("close", |_, this, ()| async move {
      let mut stream = this.stream.lock().await;
      stream.close(None).await.map_err(mlua::Error::external)?;
      Ok(())
    });
  }
}

#[derive(Debug)]
struct ServeError(String);
impl std::fmt::Display for ServeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for ServeError {}

fn to_serve_error<E: std::fmt::Display>(e: E) -> ServeError {
    ServeError(e.to_string())
}

async fn handle_request(
    lua: Arc<mlua::Lua>,
    handler_key: Arc<mlua::RegistryKey>,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, ServeError> {
    let handler = lua.registry_value::<mlua::Function>(&handler_key).map_err(to_serve_error)?;
    let (parts, body) = req.into_parts();
    let body_bytes = body.collect().await.map_err(to_serve_error)?.to_bytes();

    let req_table = lua.create_table().map_err(to_serve_error)?;
    req_table.set("method", parts.method.to_string()).map_err(to_serve_error)?;
    let uri = parts.uri.to_string();
    let host = parts
        .headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    req_table.set("host", host).map_err(to_serve_error)?;
    req_table.set("uri", uri).map_err(to_serve_error)?;

    let headers_table = lua.create_table().map_err(to_serve_error)?;
    for (k, v) in parts.headers.iter() {
        headers_table.set(k.to_string(), v.to_str().unwrap_or("")).map_err(to_serve_error)?;
    }
    req_table.set("headers", headers_table).map_err(to_serve_error)?;
    req_table.set(
        "body",
        LuluByteArray {
            bytes: body_bytes.to_vec(),
        },
    ).map_err(to_serve_error)?;

    let resp_table: mlua::Table = handler.call_async(req_table).await.map_err(to_serve_error)?;
    
    let status: u16 = resp_table.get("status").unwrap_or(200);
    let body: mlua::Value = resp_table.get("body").unwrap_or(mlua::Value::Nil);
    let headers: HashMap<String, String> = resp_table.get("headers").unwrap_or_default();

    let mut resp_builder = Response::builder()
        .status(StatusCode::from_u16(status).map_err(to_serve_error)?)
        .header("x-powered-by", "Lulu");

    for (k, v) in headers {
        resp_builder = resp_builder.header(
            HeaderName::from_bytes(k.as_bytes()).map_err(to_serve_error)?,
            HeaderValue::from_str(&v).map_err(to_serve_error)?,
        );
    }

    let body_bytes = match body {
        mlua::Value::String(s) => s.as_bytes().to_vec(),
        mlua::Value::UserData(ud) => ud.borrow::<LuluByteArray>().map_err(to_serve_error)?.bytes.clone(),
        _ => Vec::new(),
    };

    let resp = resp_builder.body(Full::new(Bytes::from(body_bytes))).map_err(to_serve_error)?;
    Ok(resp)
}

pub fn into_module(){

  create_std_module("net")
    .on_register(|lua, net_mod| {
      // HTTP client
      let http_mod = lua.create_table()?;
      let client = Client::builder()
        .user_agent("Lulu/1.0")
        .build()
        .map_err(LuaError::external)?;
      lua
        .globals()
        .set("__reqwest_client", lua.create_any_userdata(client)?)?;

      http_mod.set(
        "request",
        lua.create_async_function(|lua, req_table: mlua::Table| async move {
          let client = lua.globals().get::<mlua::AnyUserData>("__reqwest_client")?;
          let client = client.borrow::<Client>()?;

          let url: String = req_table.get("url")?;
          let method: Option<String> = req_table.get("method").ok();
          let body: Option<mlua::Value> = req_table.get("body").ok();
          let headers: Option<HashMap<String, String>> = req_table.get("headers").ok();

          let mut req = client.request(
            Method::from_bytes(method.unwrap_or_else(|| "GET".to_string()).as_bytes())
              .map_err(LuaError::external)?,
            &url,
          );

          if let Some(hmap) = headers {
            let mut hdrs = HeaderMap::new();
            for (k, v) in hmap {
              hdrs.insert(
                HeaderName::from_bytes(k.as_bytes()).map_err(mlua::Error::external)?,
                HeaderValue::from_str(&v).map_err(mlua::Error::external)?,
              );
            }
            req = req.headers(hdrs);
          }

          if let Some(body) = body {
            match body {
              mlua::Value::UserData(data) => {
                let data = data.borrow::<LuluByteArray>()?;
                req = req.body(data.bytes.clone());
              }
              mlua::Value::String(str) => req = req.body(str.to_str()?.to_string()),
              mlua::Value::Nil => {}
              _ => {
                eprintln!("Unsupported body type, only ByteArray and String is allowed.")
              }
            }
          }

          let resp = req.send().await.map_err(LuaError::external)?;
          let status = resp.status().as_u16();
          let res_headers: HashMap<String, String> = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
          let bytes = resp.bytes().await.map_err(LuaError::external)?;

          // Build Lua result
          let res = lua.create_table()?;
          res.set(
            "body",
            LuluByteArray {
              bytes: bytes.to_vec(),
            },
          )?;
          res.set("status", status)?;
          res.set("headers", res_headers)?;

          Ok(res)
        })?,
      )?;

      http_mod.set(
        "serve",
        lua.create_async_function(
          |lua, (addr, handler): (String, mlua::Function)| async move {
            let handler_key = Arc::new(lua.create_registry_value(handler)?);
            let lua_state = Arc::new(lua.clone());

            let socket_addr: SocketAddr = addr.parse().map_err(LuaError::external)?;
            let listener = tokio::net::TcpListener::bind(socket_addr)
              .await
              .map_err(LuaError::external)?;

            let server_task = tokio::spawn(async move {
                loop {
                    let (stream, _) = match listener.accept().await {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Failed to accept connection: {}", e);
                            continue;
                        }
                    };

                    let io = TokioIo::new(stream);
                    let lua_state = lua_state.clone();
                    let handler_key = handler_key.clone();

                    let service = service_fn(move |req: Request<hyper::body::Incoming>| {
                        let lua = lua_state.clone();
                        let handler = handler_key.clone();
                        async move {
                            Ok::<_, Infallible>(
                                match handle_request(lua, handler, req).await {
                                    Ok(resp) => resp,
                                    Err(e) => {
                                        eprintln!("Server error: {}", e);
                                        Response::builder()
                                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                                            .body(Full::new(Bytes::from(format!("Internal Server Error: {}", e))))
                                            .unwrap()
                                    }
                                }
                            )
                        }
                    });

                    tokio::spawn(async move {
                        if let Err(err) = http1::Builder::new()
                            .serve_connection(io, service)
                            .await
                        {
                            eprintln!("Error serving connection: {:?}", err);
                        }
                    });
                }
            });

            TOK_ASYNC_HANDLES.lock().unwrap().push(server_task);
            Ok(())
          },
        )?,
      )?;

      net_mod.set("http", http_mod)?;

      let tcp_mod = lua.create_table()?;
      tcp_mod.set(
        "connect",
        lua.create_async_function(|_, addr: String| async move {
          let stream = TcpStream::connect(addr)
            .await
            .map_err(mlua::Error::external)?;
          Ok(LuluTcpStream::new(stream))
        })?,
      )?;
      tcp_mod.set(
        "listen",
        lua.create_async_function(|_, addr: String| async move {
          let listener = TcpListener::bind(addr)
            .await
            .map_err(mlua::Error::external)?;
          Ok(LuluTcpListener {
            listener: Arc::new(listener),
          })
        })?,
      )?;
      net_mod.set("tcp", tcp_mod)?;

      let udp_mod = lua.create_table()?;
      udp_mod.set(
        "bind",
        lua.create_async_function(|_, addr: String| async move {
          let socket = UdpSocket::bind(addr).await.map_err(mlua::Error::external)?;
          Ok(LuluUdpSocket::new(socket))
        })?,
      )?;
      net_mod.set("udp", udp_mod)?;

      let ws_mod = lua.create_table()?;
      ws_mod.set(
        "connect",
        lua.create_async_function(|_, url: String| async move {
          let (ws_stream, _) = connect_async(url).await.map_err(mlua::Error::external)?;
          Ok(LuluWebSocket::new(ws_stream))
        })?,
      )?;
      net_mod.set("websocket", ws_mod)?;

      Ok(net_mod)
    })
    .add_file("net.lua", include_str!("../builtins/net/net.lua"))
    .add_file("http.lua", include_str!("../builtins/net/http.lua"))
    .add_macro(
      "error_res",
      vec!["code".into(), "message".into()],
      "Response { status = $code, body = $message }",
    )
    .add_macro(
      "json_res",
      vec!["message".into()],
      "Response { status = 200, body = serde.json.encode($message) }",
    )
    .depend_on("serde".to_string())
    .into();
}