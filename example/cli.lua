setup_downloader({
  download_text = "curl",
  progressbar_size = 10,
  progressbar_colors = { 255, 0, 0, 0, 0, 255 }
})
sync_call(function()
  mkdir("/tmp/lulu")
  download_uncached("http://localhost:3000/lulu-ui", "/tmp/lulu")
end)