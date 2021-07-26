defmodule StratepigUpdater.Updater do
  import Plug.Conn
  use Plug.Router

  plug(:match)
  plug(:dispatch)

  def init(_opts) do
    {:ok, launcher_contents} = File.read("./data/update/launcher.txt")
    {:ok, game_contents} = File.read("./data/update/game.txt")

    :ets.new(:file_storage, [:named_table])
    :ets.insert(:file_storage, {"launcher", launcher_contents})
    :ets.insert(:file_storage, {"game", game_contents})
  end

  forward "/launcher", to: StratepigUpdater.Routers.Launcher
  forward "/game", to: StratepigUpdater.Routers.Game

  match _ do
    send_resp(conn, 404, "not found")
  end
end
