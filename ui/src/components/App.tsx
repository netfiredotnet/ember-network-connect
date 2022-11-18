import React, { useState, useEffect, useRef } from "react";
import logo from "../img/logo.svg";
import { Navbar, Provider, Container, Flex, Button, Heading } from "rendition";
import { Notifications } from "./Notifications";
import { createGlobalStyle } from "styled-components";

const GlobalStyle = createGlobalStyle`
	body {
		margin: 0;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
			'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
			sans-serif;
		-webkit-font-smoothing: antialiased;
		-moz-osx-font-smoothing: grayscale;
	}

	code {
		font-family: source-code-pro, Menlo, Monaco, Consolas, 'Courier New', monospace;
	}
`;

const App = () => {
  const [attemptedReset, setAttemptedReset] = React.useState(false);
  const [error, setError] = React.useState("");
  const [timer, setTimer] = React.useState<number>(-1);

  React.useEffect(() => {
    if (timer == -1) {
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
            setTimer((t) => {
              if (t > 0) {
                return t - 1;
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

  const onReset = () => {
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
    <Provider>
      <GlobalStyle />
      <Navbar brand={<img src={logo} style={{ height: 30 }} alt="logo" />} style={{ backgroundColor: "#29292F" }} />

      <Container>
        <Notifications attemptedReset={attemptedReset} timer={timer} error={error} />
        <Flex flexDirection="column" alignItems="center" justifyContent="center" m={4} mt={5}>
          <Heading.h3 align="center" mb={4}>
            Click the below button to reset this device's network settings to DHCP. Any static IP settings will be lost.
          </Heading.h3>

          <Button onClick={onReset} danger={true}>
            Reset to DHCP
          </Button>
        </Flex>
      </Container>
    </Provider>
  );
};

export default App;
