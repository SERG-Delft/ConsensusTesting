FROM ubuntu:18.04

LABEL maintainer="email=martijnvanmeerten@hotmail.com"

RUN export LANGUAGE=C.UTF-8; export LANG=C.UTF-8; export LC_ALL=C.UTF-8; export DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
	apt-get -y upgrade && \
	apt-get -y install git pkg-config protobuf-compiler libprotobuf-dev libssl-dev wget build-essential && \
	apt-get update && \
	apt-get -y install g++-8 gcc-8 && \
	update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-7 700 --slave /usr/bin/g++ g++ /usr/bin/g++-7 && \
	update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-8 800 --slave /usr/bin/g++ g++ /usr/bin/g++-8
	
RUN wget https://github.com/Kitware/CMake/releases/download/v3.25.1/cmake-3.25.1.tar.gz && \
	tar xvzf cmake-3.25.1.tar.gz && \
	cd cmake-3.25.1 && \
	./bootstrap && \
	make && \
	make install && \
	cd .. && \
	wget https://boostorg.jfrog.io/artifactory/main/release/1.75.0/source/boost_1_75_0.tar.gz && \
	tar xvzf boost_1_75_0.tar.gz && \
	cd boost_1_75_0 && \
	./bootstrap.sh && \
	./b2 -j 4 && \
	export BOOST_ROOT=/boost_1_75_0

RUN git clone https://github.com/xrplf/rippled.git && \
	export BOOST_ROOT=/boost_1_75_0 && \
	cd rippled && \
	git checkout tags/1.7.2 && \
	sed -i '319,321d' src/ripple/overlay/impl/Handshake.cpp && \
	mkdir my_build && \
	cd my_build && \
	cmake .. && \
	cmake --build . -- -j 4

FROM ubuntu:18.04
COPY --from=0 /rippled/my_build/rippled /rippled/my_build/rippled

ENTRYPOINT [ "/rippled/my_build/rippled" ]
