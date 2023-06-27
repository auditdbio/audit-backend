FROM blang/latex:ubuntu

RUN apt-get update
RUN apt-get install ca-certificates -y
RUN apt-get install pandoc -y
WORKDIR /backend
COPY ./setup.sh .
RUN chmod +x ./setup.sh
CMD ["./setup.sh"]
