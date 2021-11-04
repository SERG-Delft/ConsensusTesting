if ($args[1] -eq "p") {
  docker ps -a --filter "ancestor=mvanmeerten/rippled-boost-cmake" --format="{{.ID}}" | ForEach-Object -Parallel {docker stop $_; docker rm $_}
} else {
  docker ps -a --filter "ancestor=mvanmeerten/rippled-boost-cmake" --format="{{.ID}}" | ForEach-Object -Process {docker stop $_; docker rm $_}
}