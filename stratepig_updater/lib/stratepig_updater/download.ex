defmodule StratepigUpdater.Updater.Download do
  import Plug.Conn

  @spec stream_file(Plug.Conn.t(), String.t(), String.t()) :: any()
  def stream_file(conn, path, display_name) do
    %{size: size} = File.stat!("#{path}")
    file_stream = File.stream!("#{path}", [], 200)

    conn =
      conn
      |> put_resp_header("X-Content-Length", Integer.to_string(size, 10))
      |> put_resp_header("Content-disposition", "attachment; filename=\"#{display_name}\"")
      |> put_resp_header("X-Accel-Redirect", "/tempfile/download/#{display_name}")
      |> put_resp_header("Content-Type", "application/octet-stream")
      |> send_chunked(200)

    file_stream
    |> Enum.reduce_while(conn, fn chunk, conn ->
      case chunk(conn, chunk) do
        {:ok, conn} ->
          {:cont, conn}

        {:error, :closed} ->
          {:halt, conn}
      end
    end)
  end
end
