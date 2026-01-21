import type { FC } from "react";
import { AlertCircle, CheckCircle, Clock, AlertTriangle } from "lucide-react";

interface NotificationsProps {
  attemptedReset: boolean;
  timer: number;
  error: string;
}

export const Notifications: FC<NotificationsProps> = ({
  attemptedReset,
  timer,
  error,
}) => {
  return (
    <div className="space-y-3">
      {attemptedReset && (
        <div className="flex items-start gap-3 p-4 rounded-lg bg-blue-50 border border-blue-200 text-blue-800 dark:bg-blue-950 dark:border-blue-800 dark:text-blue-200">
          <CheckCircle className="h-5 w-5 mt-0.5 shrink-0" />
          <div>
            <span className="font-medium">Applying changes... </span>
            <span>
              Please ask NetFire staff to check connectivity to the device. If
              you need to make any port changes or other network config changes,
              you may do so now.
            </span>
          </div>
        </div>
      )}

      {!attemptedReset && timer > 0 && (
        <div className="flex items-start gap-3 p-4 rounded-lg bg-amber-50 border border-amber-200 text-amber-800 dark:bg-amber-950 dark:border-amber-800 dark:text-amber-200">
          <Clock className="h-5 w-5 mt-0.5 shrink-0" />
          <div>
            <span className="font-medium">
              This access point will shut down in {timer} seconds.{" "}
            </span>
            <span>
              If you still need more time, you can power the device off and on
              again, after which point the countdown will restart.
            </span>
          </div>
        </div>
      )}

      {!attemptedReset && timer === 0 && (
        <div className="flex items-start gap-3 p-4 rounded-lg bg-amber-100 border-2 border-amber-300 text-amber-900 dark:bg-amber-900 dark:border-amber-700 dark:text-amber-100">
          <AlertTriangle className="h-5 w-5 mt-0.5 shrink-0" />
          <span className="font-semibold">
            Access point is shutting down now!
          </span>
        </div>
      )}

      {!!error && (
        <div className="flex items-start gap-3 p-4 rounded-lg bg-red-50 border border-red-200 text-red-800 dark:bg-red-950 dark:border-red-800 dark:text-red-200">
          <AlertCircle className="h-5 w-5 mt-0.5 shrink-0" />
          <span>{error}</span>
        </div>
      )}
    </div>
  );
};
