defmodule StratepigUpdater.Routers.Launcher do
  use Plug.Router
  import Plug.Conn

  alias StratepigUpdater.Updater.Download

  plug :match
  plug :dispatch

  get "/version" do
    send_resp(conn, 200, StratepigUpdater.MixProject.launcher_version())
  end

  get "/update" do
    [{"launcher", contents}] = :ets.lookup(:file_storage, "launcher")
    put_resp_content_type(conn, "text/plain")
    send_resp(conn, 200, contents)
  end

  get "/d" do
    Download.stream_file(conn, "./priv/static/hello.dap", "hello.dap")
  end

  match _ do
    send_resp(conn, 404, "not found")
  end
end
