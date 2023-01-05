build:
	docker build -t ripple -f rippled.Dockerfile .
	DOCKER_BUILDKIT=1 docker build -t byzzfuzz .

run: build
	docker run -v /var/run/docker.sock:/var/run/docker.sock -i --init --rm --net host --name byzzfuzz byzzfuzz
