defmodule StratepigUpdater.Routers.Game do
  use Plug.Router
  import Plug.Conn

  alias StratepigUpdater.Updater.Download

  plug :match
  plug :dispatch

  get "/version" do
    send_resp(conn, 200, StratepigUpdater.MixProject.version())
  end

  get "/update" do
    [{"game", contents}] = :ets.lookup(:file_storage, "game")
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
