#! /bin/bash

DEST="/mongo_backup/$(date +%F_%R)"
mongodump -o $DEST

# Delete backups created 5 or more days ago
find /mongo_backup -mindepth 1 -ctime +5 -delete