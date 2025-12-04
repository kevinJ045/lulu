
in local route do
  function GET()
    return into_response "Hello"
  end

  function POST(req)
    return into_response f"body: {req.body:to_string()}"
  end

end

return route
