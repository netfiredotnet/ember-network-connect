import { useState, useEffect } from "react";
import logo from "../img/logo.svg";
import { Button } from "@/components/ui/button";
import { Notifications } from "./Notifications";

const App = () => {
  const [attemptedReset, setAttemptedReset] = useState(false);
  const [error, setError] = useState("");
  const [timer, setTimer] = useState<number>(-1);

  useEffect(() => {
    if (timer === -1) {
      fetch("/get_timer")
        .then((data) => {
          if (data.status !== 200) {
            throw new Error(data.statusText);
          }
          return data.text();
        })
        .then((t) => {
          setTimer(parseInt(t));
          const interval = setInterval(() => {
            setTimer((prev) => {
              if (prev > 0) {
                return prev - 1;
              } else {
                clearInterval(interval);
                return 0;
              }
            });
          }, 1000);
        })
        .catch((e: Error) => {
          setError(`Failed to fetch timer. ${e.message || e}`);
        });
    }
  }, [timer]);

  const handleReset = () => {
    setAttemptedReset(true);
    setError("");

    fetch("/reset_dhcp", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
    })
      .then((resp) => {
        if (resp.status !== 200) {
          throw new Error(resp.statusText);
        }
      })
      .catch((e: Error) => {
        setError(`Failed to reset DHCP. ${e.message || e}`);
      });
  };

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <nav className="bg-[#29292F] px-4 py-3">
        <img src={logo} className="h-8" alt="Ember Network Connect" />
      </nav>

      {/* Main content */}
      <main className="container mx-auto px-4 py-8">
        <Notifications
          attemptedReset={attemptedReset}
          timer={timer}
          error={error}
        />

        <div className="flex flex-col items-center justify-center mt-8">
          <h3 className="text-xl font-medium text-center mb-6 max-w-lg">
            Click the below button to reset this device's network settings to
            DHCP. Any static IP settings will be lost.
          </h3>

          <Button variant="destructive" size="lg" onClick={handleReset}>
            Reset to DHCP
          </Button>
        </div>
      </main>
    </div>
  );
};

export default App;
