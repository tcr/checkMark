# select image
FROM rust:1.33

RUN apt-get update -y \
  && apt-get install -y apt-transport-https ca-certificates curl gnupg2 software-properties-common
RUN curl -fsSL https://download.docker.com/linux/debian/gpg | apt-key add -
RUN add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/debian $(lsb_release -cs) stable"
RUN apt-get update -y

# copy your source tree
COPY ./ /app

WORKDIR "/app"

ENV RUST_BACKTRACE=1

RUN apt-get install pdf2svg -y

CMD cargo run --release --target x86_64-unknown-linux-gnu --bin api_remarkable
