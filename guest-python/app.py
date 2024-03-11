import spin_cron
from spin_cron.imports import cron_types
from spin_sdk import variables

class SpinCron(spin_cron.SpinCron):
    def handle_cron_event(self, metadata: cron_types.Metadata) -> None:
        temp = variables.get("something")
        print("[" + str(metadata.timestamp) +"] " + "Hello every " + temp)