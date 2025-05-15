// Test an MCP server

const baseUrl = "http://127.0.0.1:3000/api/v1/servers/context7/sse"

/**
 * Log the SSE stream continuously and resolve when the endpoint is found but
 * keep logging the rest of the stream.
 * 
 * @param stream The SSE stream to log
 * @returns The endpoint URL
 */
async function printSse(stream: ReadableStream<Uint8Array>): Promise<string> {
  const decoder = new TextDecoder();
  const reader = stream.getReader();
  let buffer = "";
  
  return new Promise<string>(async(resolve) => {
      try {
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          const text = decoder.decode(value);
          console.log(`\n${text}`);
          
          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split('\n');
          buffer = lines.pop() || ""; // Keep the last partial line
          
          for (let i = 0; i < lines.length; i++) {
            if (lines[i].startsWith("event: endpoint") && i + 1 < lines.length) {
              const dataLine = lines[i + 1];
              if (dataLine.startsWith("data: ")) {
                const endpoint = dataLine.substring(6);
                resolve(endpoint);
                // Continue processing the stream even after finding the endpoint
                i++; // Skip the data line as we've already processed it
              }
            }
          }
        }
      } catch (error) {
        console.error("Error reading SSE stream:", error);
      }
  });
}

function initialize(endpoint: string) {
  const url = new URL(endpoint, baseUrl);
  return fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      "id":0,
      "jsonrpc":"2.0",
      "method":"initialize",
      "params":{
        "protocolVersion":"2024-11-05",
        "capabilities":{},
        "clientInfo":{
          "name":"SCRIPT",
          "version":"0.11.0"
        },
      }
    }),
  }).then(async(res) => {
    if (!res.ok) {
      throw new Error(`[ERROR/${res.statusText}] ${await res.text()}`);
    }
    return res.json();
  })
}

function listTools(endpoint: string) {
  const url = new URL(endpoint, baseUrl);
  return fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      "id":0,
      "jsonrpc":"2.0",
      "method":"tools/list",
      "params":{}
    }),
  }).then(async(res) => {
    if (!res.ok) throw new Error(`Failed to list tools: ${res.status} ${await res.text()}`);
    const tools = await res.json();
    return tools.result.tools.map(x => x.name)
  });
}

async function main() {
  const sse = await fetch(baseUrl).then(async(res) => res.body);
  if (!sse) throw new Error("No response body");
  const endpoint = await printSse(sse);

  console.log(`\n---------------------------`);
  console.log(await initialize(endpoint));
  console.log(`\n---------------------------`);
  console.log(await listTools(endpoint));
  console.log(`\n---------------------------`);
}

main()
  .catch((err) => {
    console.error(err);
  });
