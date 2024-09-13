#!/bin/bash

XML_FILE="/data/data/com.dragon.read/shared_prefs/applog_stats.xml"

if [ ! -f "$XML_FILE" ]; then
  echo "文件不存在: $XML_FILE"
  exit 1
fi

device_id=$(grep '<string name="device_id">' "$XML_FILE" | sed 's/.*<string name="device_id">//' | sed 's/<\/string>//')

install_id=$(grep '<string name="install_id">' "$XML_FILE" | sed 's/.*<string name="install_id">//' | sed 's/<\/string>//')

echo "Device ID: $device_id"
echo "Install ID: $install_id"
