defmodule StratepigUpdater.Utils.Files do
  @type updatable :: :launcher | :game

  def priv_dir() do
    "/files/priv"
  end

  def static_asset(path) do
    Path.join(priv_dir(), "/static/#{path}")
  end

  @spec update_file(updatable) :: any()
  def update_file(typ) do
    case typ do
      :launcher -> static_asset("updates/launcher.txt")
      :game -> static_asset("updates/game.txt")
    end
  end

  @spec version_file(updatable) :: any()
  def version_file(typ) do
    case typ do
      :launcher -> static_asset("versions/launcher.txt")
      :game -> static_asset("versions/game.txt")
    end
  end

  @spec binary(updatable) :: any()
  def binary(typ) do
    case typ do
      :launcher -> static_asset("bin/Launcher.zip")
      :game -> static_asset("bin/Build.zip")
    end
  end
end
