import * as React from "react";
import { Txt, Alert } from "rendition";

export const Notifications = ({
  attemptedReset,
  timer,
  error,
}: {
  attemptedReset: boolean;
  timer: number;
  error: string;
}) => {
  return (
    <>
      {attemptedReset && (
        <Alert m={2} info>
          <Txt.span>Applying changes... </Txt.span>
          <Txt.span>
            Please ask NetFire staff to check connectivity to the device. If you need to make any port changes or other
            network config changes, you may do so now.
          </Txt.span>
        </Alert>
      )}
      {!attemptedReset && timer > 0 && (
        <Alert m={2} warning>
          <Txt.span>This access point will shut down in {timer} seconds.&nbsp;</Txt.span>
          <Txt.span>
            If you still need more time, you can power the device off an on again, after which point the countdown will
            restart.
          </Txt.span>
        </Alert>
      )}
      {!attemptedReset && timer == 0 && (
        <Alert m={2} emphasized warning>
          <Txt.span>Access point is shutting down now!</Txt.span>
        </Alert>
      )}
      {!!error && (
        <Alert m={2} danger>
          <Txt.span>{error}</Txt.span>
        </Alert>
      )}
    </>
  );
};
