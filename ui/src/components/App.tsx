import { Button } from "@/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { Spinner } from "@/components/ui/spinner";
import { AlertTriangle, CheckCircle, Clock } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast, Toaster } from "sonner";
import logo from "../img/logo.svg";

const App = () => {
  const { t } = useTranslation();
  const [timer, setTimer] = useState<number>(-1);
  const [isResetting, setIsResetting] = useState(false);
  const [resetSuccess, setResetSuccess] = useState(false);
  const intervalRef = useRef<number | null>(null);

  // Fetch initial timer value
  useEffect(() => {
    fetch("/get_timer")
      .then((data) => {
        if (data.status !== 200) {
          throw new Error(data.statusText);
        }
        return data.text();
      })
      .then((t) => {
        setTimer(parseInt(t));
      })
      .catch((e: Error) => {
        toast.error(t("errors.fetchTimer"), {
          description: e.message || String(e),
        });
      });
  }, [t]);

  // Start countdown interval when timer becomes positive
  useEffect(() => {
    // Only start interval if timer is positive and no interval is running
    if (timer <= 0 || intervalRef.current !== null) return;

    intervalRef.current = window.setInterval(() => {
      setTimer((prev) => {
        if (prev > 1) {
          return prev - 1;
        } else {
          if (intervalRef.current) {
            clearInterval(intervalRef.current);
            intervalRef.current = null;
          }
          return 0;
        }
      });
    }, 1000);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, [timer]);

  const handleReset = () => {
    setIsResetting(true);

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
        setResetSuccess(true);
      })
      .catch((e: Error) => {
        setIsResetting(false);
        toast.error(t("errors.resetDhcp"), {
          description: e.message || String(e),
        });
      });
  };

  // Determine which state to show
  const showLoading = timer === -1;
  const showSuccess = resetSuccess;
  const showShutdown = !resetSuccess && timer === 0;
  const showNormal = !resetSuccess && timer > 0;

  return (
    <div className='min-h-screen bg-background'>
      <Toaster richColors position='top-center' />

      {/* Header */}
      <nav className='bg-[#29292F] px-4 py-3'>
        <img src={logo} className='h-8' alt={t("app.logoAlt")} />
      </nav>

      {/* Main content */}
      <main className='container mx-auto px-4 py-8'>
        {/* Loading state */}
        {showLoading && (
          <Empty className='w-full mt-12'>
            <EmptyHeader>
              <EmptyMedia variant='icon'>
                <Spinner className='size-6' />
              </EmptyMedia>
              <EmptyTitle>{t("loading.title")}</EmptyTitle>
              <EmptyDescription>{t("loading.description")}</EmptyDescription>
            </EmptyHeader>
          </Empty>
        )}

        {/* Success state - replaces all content */}
        {showSuccess && (
          <div className='flex flex-col items-center justify-center mt-12'>
            <div className='flex items-center gap-3 p-6 rounded-lg bg-green-50 border border-green-200 text-green-800 dark:bg-green-950 dark:border-green-800 dark:text-green-200 max-w-lg'>
              <CheckCircle className='h-8 w-8 shrink-0' />
              <div>
                <p className='font-semibold text-lg'>{t("success.title")}</p>
                <p className='mt-1'>{t("success.description")}</p>
              </div>
            </div>
          </div>
        )}

        {/* Shutdown state - replaces all content */}
        {showShutdown && (
          <div className='flex flex-col items-center justify-center mt-12'>
            <div className='flex items-center gap-3 p-6 rounded-lg bg-amber-100 border-2 border-amber-300 text-amber-900 dark:bg-amber-900 dark:border-amber-700 dark:text-amber-100 max-w-lg'>
              <AlertTriangle className='h-8 w-8 shrink-0' />
              <div>
                <p className='font-semibold text-lg'>{t("shutdown.title")}</p>
                <p className='mt-1'>{t("shutdown.description")}</p>
              </div>
            </div>
          </div>
        )}

        {/* Normal state - countdown and button */}
        {showNormal && (
          <>
            {/* Countdown timer */}
            <div className='flex items-center justify-center gap-2 p-4 mb-6 rounded-lg bg-amber-50 border border-amber-200 text-amber-800 dark:bg-amber-950 dark:border-amber-800 dark:text-amber-200'>
              <Clock className='h-5 w-5 shrink-0' />
              <span>
                <span className='font-medium'>{t("countdown.message", { seconds: timer })}</span> {t("countdown.hint")}
              </span>
            </div>

            <div className='flex flex-col items-center justify-center mt-8'>
              <h3 className='text-xl font-medium text-center mb-6 max-w-lg'>{t("reset.heading")}</h3>

              <Button variant='destructive' size='lg' onClick={handleReset} disabled={isResetting}>
                {isResetting ? t("reset.buttonLoading") : t("reset.button")}
              </Button>
            </div>
          </>
        )}
      </main>
    </div>
  );
};

export default App;
