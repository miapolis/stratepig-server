defmodule StratepigUpdater.MixProject do
  use Mix.Project

  def project do
    [
      app: :stratepig_updater,
      version: "0.7.1",
      elixir: ">= 1.11.3",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  def application do
    [
      extra_applications: [:logger],
      mod: {StratepigUpdater.Application, []}
    ]
  end

  defp deps do
    [
      {:plug_cowboy, "~> 2.0"}
    ]
  end
end
