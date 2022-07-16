if ($args[1] -eq "p") {
  docker ps -a --filter "name=validator" --format="{{.ID}}" | ForEach-Object -Parallel {docker stop $_; docker rm $_}
} else {
  docker ps -a --filter "name=validator" --format="{{.ID}}" | ForEach-Object -Process {docker stop $_; docker rm $_}
}