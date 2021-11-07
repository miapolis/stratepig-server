defmodule StratepigUpdater.Routers.Launcher do
  use Plug.Router
  import Plug.Conn

  alias StratepigUpdater.Updater
  alias StratepigUpdater.Updater.Download
  alias StratepigUpdater.Utils.Version
  alias StratepigUpdater.Utils.Files

  plug(:match)
  plug(:dispatch)

  get "/version" do
    send_resp(conn, 200, Version.launcher_version())
  end

  get "/update" do
    [{"launcher", contents}] = :ets.lookup(:file_storage, "launcher")
    put_resp_content_type(conn, "text/plain")
    send_resp(conn, 200, contents)
  end

  get "/:platform/d" do
    platform = conn.params["platform"]

    case Enum.member?(Updater.supported_platforms(), platform) do
      true ->
        Download.stream_file(conn, Files.binary(:game, platform), "launcher.zip")

      _ ->
        send_resp(conn, 404, "Platform not supported")
    end
  end

  match _ do
    send_resp(conn, 404, "not found")
  end
end
