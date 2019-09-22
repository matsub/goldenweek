FROM debian:jessie

ENV DEBIAN_FRONTEND=noninteractive

RUN apt update && apt install -y coturn

EXPOSE 3478/udp

CMD ["/usr/bin/turnserver", "--stun-only"]
