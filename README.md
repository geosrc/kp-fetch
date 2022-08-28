# kp-fetch

A simple command line tool to download geomagnetic activity [kp and ap nowcast files](https://www-app3.gfz-potsdam.de/kp_index/Kp_ap_nowcast.txt) from [GFZ](https://www.gfz-potsdam.de/en/kp-index).
It can detect the updated records since the last download and return them in a InfluxDB [Line protocol](https://docs.influxdata.com/influxdb/cloud/reference/syntax/line-protocol/) format.

The tool is meant to be executed regularly the InfluxDB [Telegraf](https://www.influxdata.com/time-series-platform/telegraf/) agent.
