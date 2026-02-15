import { useState } from "react";
import { Box, Text, useInput } from "ink";
import { spawn } from "child_process";

interface Process {
  pid: number;
  name: string;
  protocol: string;
  localAddress: string;
  state: string;
  isAdmin?: boolean;
}

export function PortKiller() {
  const [port, setPort] = useState("");
  const [processes, setProcesses] = useState<Process[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [status, setStatus] = useState("Please enter a port and press Enter");
  const [isSearching, setIsSearching] = useState(false);
  const [inputMode, setInputMode] = useState(true);

  useInput((input, key) => {
    if (inputMode) {
      // Port eingeben
      if (key.return && port) {
        searchProcesses();
      } else if (key.backspace || key.delete) {
        setPort(port.slice(0, -1));
      } else if (input.match(/[0-9]/)) {
        setPort(port + input);
      } else if (key.escape || input === "q") {
        process.exit(0);
      }
    } else {
      // Process selection
      if (key.upArrow) {
        setSelectedIndex(Math.max(0, selectedIndex - 1));
      } else if (key.downArrow) {
        setSelectedIndex(Math.min(processes.length - 1, selectedIndex + 1));
      } else if (key.return && processes[selectedIndex]) {
        killProcess(processes[selectedIndex]);
      } else if (key.escape) {
        setInputMode(true);
        setPort("");
        setProcesses([]);
        setSelectedIndex(0);
        setStatus("Please enter a port and press Enter");
      } else if (input === "q") {
        process.exit(0);
      }
    }
  });

  const searchProcesses = async () => {
    setIsSearching(true);
    setInputMode(false);
    setStatus(`Searching for processes on port ${port}...`);

    try {
      const foundProcesses = await findProcessesByPort(parseInt(port));
      setProcesses(foundProcesses);

      if (foundProcesses.length === 0) {
        setStatus(`No processes found on port ${port}. ESC for new search.`);
      } else {
        setStatus(
          `${foundProcesses.length} process(es) found. ↑↓ to select, Enter to kill, ESC to go back.`,
        );
      }
    } catch (error) {
      setStatus(`Error: ${error}. ESC for new search.`);
    }
    setIsSearching(false);
  };

  const killProcess = async (process: Process) => {
    if (process.isAdmin) {
      setStatus(
        `Process ${process.name} (PID: ${process.pid}) is an admin process and cannot be killed!`,
      );
      return;
    }

    setStatus(`Killing process ${process.name} (PID: ${process.pid})...`);

    try {
      await killProcessByPid(process.pid);
      setStatus(`Process ${process.name} (PID: ${process.pid}) was killed!`);

      // Refresh the list
      setTimeout(() => searchProcesses(), 1000);
    } catch (error) {
      setStatus(`Error killing process: ${error}`);
    }
  };

  return (
    <Box flexDirection="column" padding={1} width={100}>
      {/* Header */}
      <Box
        borderStyle="round"
        borderColor="cyan"
        paddingX={2}
        paddingY={0}
        marginBottom={1}
      >
        <Text bold color="cyan">
          PORT KILLER
        </Text>
        <Text color="gray"> - Find and Kill Processes</Text>
      </Box>

      {inputMode ? (
        // Input Mode
        <Box flexDirection="column">
          <Box
            borderStyle="single"
            borderColor="yellow"
            paddingX={2}
            paddingY={1}
            flexDirection="column"
          >
            <Box marginBottom={1}>
              <Text bold color="yellow">
                PORT INPUT
              </Text>
            </Box>
            <Box>
              <Text color="white">Port: </Text>
              <Text bold color="yellow">
                {port}
              </Text>
              <Text color="yellow">_</Text>
            </Box>
          </Box>

          <Box marginTop={1} paddingX={1}>
            <Text dimColor>
              💡 Tip: Enter a port number (e.g. 3000) and press{" "}
            </Text>
            <Text bold color="green">
              ↵ Enter
            </Text>
          </Box>
        </Box>
      ) : (
        // Process List Mode
        <Box flexDirection="column">
          {/* Port Info Box */}
          <Box
            borderStyle="single"
            borderColor="green"
            paddingX={2}
            paddingY={0}
            marginBottom={1}
          >
            <Text>Found processes on port: </Text>
            <Text bold color="green">
              {port}
            </Text>
            <Text color="gray">
              {" "}
              ({processes.length} process{processes.length !== 1 ? "es" : ""})
            </Text>
          </Box>

          {isSearching ? (
            <Box
              borderStyle="round"
              borderColor="yellow"
              paddingX={2}
              paddingY={1}
              justifyContent="center"
            >
              <Text color="yellow">⏳ Searching network connections...</Text>
            </Box>
          ) : processes.length === 0 ? (
            <Box
              borderStyle="round"
              borderColor="gray"
              paddingX={2}
              paddingY={1}
              flexDirection="column"
              alignItems="center"
            >
              <Text color="gray">No processes found</Text>
              <Text dimColor>Port {port} is not used by any process</Text>
            </Box>
          ) : (
            <Box
              borderStyle="single"
              borderColor="cyan"
              paddingX={1}
              paddingY={1}
              flexDirection="column"
            >
              {/* Table Header */}
              <Box marginBottom={1}>
                <Box width={4}>
                  <Text bold color="cyan">
                    {" "}
                  </Text>
                </Box>
                <Box width={25}>
                  <Text bold color="cyan">
                    PROCESS
                  </Text>
                </Box>
                <Box width={10}>
                  <Text bold color="cyan">
                    PID
                  </Text>
                </Box>
                <Box width={12}>
                  <Text bold color="cyan">
                    PROTOCOL
                  </Text>
                </Box>
                <Box width={25}>
                  <Text bold color="cyan">
                    ADDRESS
                  </Text>
                </Box>
                <Box width={15}>
                  <Text bold color="cyan">
                    STATE
                  </Text>
                </Box>
              </Box>

              {/* Process List */}
              {processes.map((proc, index) => {
                const isSelected = index === selectedIndex;
                return (
                  <Box
                    key={proc.pid}
                    backgroundColor={isSelected ? "blue" : undefined}
                    paddingX={1}
                  >
                    <Box width={4}>
                      <Text color={isSelected ? "white" : "gray"}>
                        {isSelected ? "►" : " "}
                      </Text>
                    </Box>
                    <Box width={25}>
                      <Text
                        bold={isSelected}
                        color={
                          isSelected ? "white" : proc.isAdmin ? "red" : "white"
                        }
                      >
                        {proc.name.substring(0, 23)}
                        {proc.isAdmin ? " 🔒" : ""}
                      </Text>
                    </Box>
                    <Box width={10}>
                      <Text color={isSelected ? "white" : "yellow"}>
                        {proc.pid}
                      </Text>
                    </Box>
                    <Box width={12}>
                      <Text color={isSelected ? "white" : "green"}>
                        {proc.protocol}
                      </Text>
                    </Box>
                    <Box width={25}>
                      <Text color={isSelected ? "white" : "cyan"}>
                        {proc.localAddress.substring(0, 23)}
                      </Text>
                    </Box>
                    <Box width={15}>
                      <Text color={isSelected ? "white" : "magenta"}>
                        {proc.state}
                      </Text>
                    </Box>
                  </Box>
                );
              })}
            </Box>
          )}
        </Box>
      )}

      {/* Controls */}
      <Box marginTop={1} paddingX={1} flexDirection="column">
        {!inputMode && processes.length > 0 && (
          <Box marginBottom={0}>
            <Text color="cyan">↑↓</Text>
            <Text dimColor> Navigation </Text>
            <Text color="green">↵ Enter</Text>
            <Text dimColor> Kill process </Text>
            <Text color="yellow">ESC</Text>
            <Text dimColor> New search </Text>
            <Text color="red">Q</Text>
            <Text dimColor> Quit</Text>
          </Box>
        )}
        {!inputMode && processes.length === 0 && (
          <Box>
            <Text color="yellow">ESC</Text>
            <Text dimColor> New search </Text>
            <Text color="red">Q</Text>
            <Text dimColor> Quit</Text>
          </Box>
        )}
        {inputMode && (
          <Box>
            <Text color="yellow">ESC</Text>
            <Text dimColor> or </Text>
            <Text color="red">Q</Text>
            <Text dimColor> to quit</Text>
          </Box>
        )}
      </Box>
    </Box>
  );
}

function findProcessesByPort(port: number): Promise<Process[]> {
  return new Promise((resolve, reject) => {
    // Verwende netstat auf Windows
    const netstat = spawn("netstat", ["-ano"]);
    let output = "";

    netstat.stdout.on("data", (data) => {
      output += data.toString();
    });

    netstat.stderr.on("data", (data) => {
      reject(data.toString());
    });

    netstat.on("close", async (code) => {
      if (code !== 0) {
        reject(`netstat exited with code ${code}`);
        return;
      }

      const lines = output.split("\n");
      const processes: Process[] = [];
      const pidSet = new Set<number>();

      for (const line of lines) {
        // Search for lines containing the port
        if (line.includes(`:${port} `)) {
          const parts = line.trim().split(/\s+/);

          // Format: Protocol Local-Address Foreign-Address State PID
          if (parts.length >= 5) {
            const lastPart = parts[parts.length - 1];
            const pid = parseInt(lastPart || "0");
            if (!isNaN(pid) && !pidSet.has(pid)) {
              pidSet.add(pid);

              const processName = await getProcessName(pid);
              const isAdmin = await isAdminProcess(pid);

              // Get state and translate German state names to English
              let rawState = parts[3] || "LISTENING";
              const stateTranslations: Record<string, string> = {
                ABHÖREN: "LISTENING",
                "ABH�REN": "LISTENING", // For encoding issues
                HERGESTELLT: "ESTABLISHED",
                WARTEND: "TIME_WAIT",
                SCHLIEßEN: "CLOSE_WAIT",
                "SCHLIE�EN": "CLOSE_WAIT", // For encoding issues
                FIN_WARTEN_1: "FIN_WAIT_1",
                FIN_WARTEN_2: "FIN_WAIT_2",
                GESCHLOSSEN: "CLOSED",
              };
              const state = stateTranslations[rawState.trim()] || rawState;

              processes.push({
                pid,
                name: processName,
                protocol: parts[0] || "UNKNOWN",
                localAddress: parts[1] || "UNKNOWN",
                state,
                isAdmin,
              });
            }
          }
        }
      }

      resolve(processes);
    });
  });
}

function getProcessName(pid: number): Promise<string> {
  return new Promise((resolve) => {
    const tasklist = spawn("tasklist", [
      "/FI",
      `PID eq ${pid}`,
      "/FO",
      "CSV",
      "/NH",
    ]);
    let output = "";

    tasklist.stdout.on("data", (data) => {
      output += data.toString();
    });

    tasklist.on("close", () => {
      // Parse CSV output
      const match = output.match(/"([^"]+)"/);
      resolve(match && match[1] ? match[1] : `Process-${pid}`);
    });
  });
}

function isAdminProcess(pid: number): Promise<boolean> {
  return new Promise((resolve) => {
    // Try to read process info - if we don't have permission, it's probably an admin process
    const wmic = spawn("wmic", [
      "process",
      "where",
      `ProcessId=${pid}`,
      "get",
      "ExecutablePath",
    ]);
    let hasError = false;

    wmic.stderr.on("data", () => {
      hasError = true;
    });

    wmic.on("close", () => {
      // If we have an error or certain system PIDs, it's an admin process
      if (hasError || pid < 1000) {
        resolve(true);
      } else {
        resolve(false);
      }
    });
  });
}

function killProcessByPid(pid: number): Promise<void> {
  return new Promise((resolve, reject) => {
    const taskkill = spawn("taskkill", ["/PID", pid.toString(), "/F"]);

    taskkill.on("close", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(`taskkill exited with code ${code}`);
      }
    });

    taskkill.stderr.on("data", (data) => {
      reject(data.toString());
    });
  });
}
