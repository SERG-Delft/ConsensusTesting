FROM ubuntu:18.04

LABEL maintainer="email=martijnvanmeerten@hotmail.com"

RUN export LANGUAGE=C.UTF-8; export LANG=C.UTF-8; export LC_ALL=C.UTF-8; export DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
	apt-get -y upgrade && \
	apt-get -y install git pkg-config protobuf-compiler libprotobuf-dev libssl-dev wget build-essential
	
RUN wget https://github.com/Kitware/CMake/releases/download/v3.13.3/cmake-3.13.3-Linux-x86_64.sh && \
	sh cmake-3.13.3-Linux-x86_64.sh --prefix=/usr/local --exclude-subdir && \
	wget https://boostorg.jfrog.io/artifactory/main/release/1.75.0/source/boost_1_75_0.tar.gz && \
	tar xvzf boost_1_75_0.tar.gz && \
	cd boost_1_75_0 && \
	./bootstrap.sh && \
	./b2 -j 4 && \
	export BOOST_ROOT=/boost_1_75_0
