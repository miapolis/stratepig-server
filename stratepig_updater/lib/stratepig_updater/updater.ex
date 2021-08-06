defmodule StratepigUpdater.Updater do
  import Plug.Conn
  use Plug.Router

  alias StratepigUpdater.Utils.Files
  alias StratepigUpdater.Utils.Version

  plug(:match)
  plug(:dispatch)

  def init(opts) do
    IO.puts("Starting Stratepig Updater...")

    {:ok, launcher_contents} = File.read(Files.update_file(:launcher))
    {:ok, game_contents} = File.read(Files.update_file(:game))

    :ets.new(:file_storage, [:named_table])
    :ets.insert(:file_storage, {"launcher", launcher_contents})
    :ets.insert(:file_storage, {"game", game_contents})

    Version.init(opts)
  end

  forward("/launcher", to: StratepigUpdater.Routers.Launcher)
  forward("/game", to: StratepigUpdater.Routers.Game)

  match _ do
    send_resp(conn, 404, "not found")
  end
end
