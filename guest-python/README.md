# Python Sample for Spin Cron Trigger

## Setup Environment and Dependencies

```bash
python3 -m venv venv
source venv/bin/activate
pip3 install -r requirements.txt
```

To generate bindings to use with intellisense

```bash
componentize-py -d ../cron.wit -w spin-cron bindings bindings
mv bindings/spin_cron ./
rm -r bindings
```


## Build and run the app

```bash
$ spin up --build 
[1710200677] Hello every 2s
[1710200679] Hello every 2s
```
