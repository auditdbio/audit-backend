FROM mongo:latest
RUN apt-get update && apt-get install -y cron
COPY auto_dump.sh /auto_dump.sh
COPY crontab.txt /crontab.txt
RUN chmod +x /auto_dump.sh
RUN crontab crontab.txt
RUN service cron restart
