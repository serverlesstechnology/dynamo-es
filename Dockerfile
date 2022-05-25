FROM rust:1.60 as builder

WORKDIR /home/build
RUN git clone https://github.com/serverlesstechnology/dynamo-es.git
WORKDIR /home/build/dynamo-es