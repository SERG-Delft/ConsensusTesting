docker ps --filter "ancestor=mvanmeerten/rippled-boost-cmake" --format="{{.ID}}" | ForEach-Object -Process {docker stop $_; docker rm $_}