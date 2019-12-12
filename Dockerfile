# based on ubuntu image
FROM ubuntu:latest
# Install dependencies
USER root
RUN apt-get -q update && apt-get -qqy upgrade && apt-get -qqy install curl gcc g++ build-essential make cmake libssl-dev && \
curl -sL https://deb.nodesource.com/setup_12.x | bash - && \
apt-get -qqy install git nodejs sudo ssh && apt-get -y autoremove && \
apt-get clean && npm install -g node-gyp && rm -rf /var/lib/apt/lists/*

# RUN adjust user and group
RUN addgroup --gid 999 jenkins && \
adduser --quiet --disabled-password --home /home/jenkins --shell /bin/sh --gid 999 jenkins && \
usermod -aG sudo jenkins && echo "%sudo	ALL=(ALL:ALL) ALL" >> /etc/sudoers && mkdir /home/jenkins/.ssh

# switch user
WORKDIR /home/jenkins
USER jenkins

# ssh known hosts
RUN ssh-keyscan -H github.com

# install rust
RUN curl https://sh.rustup.rs -sSf > $HOME/rustup-init && chmod +x $HOME/rustup-init && $HOME/rustup-init -y
ENV PATH="/home/jenkins/.cargo/bin:${PATH}"
RUN rustup component add rustfmt-preview && rustup target add i686-unknown-linux-gnu

# 
CMD ["sh"]
