import { createServer } from "node:http";
import { readFileSync } from "node:fs";

// Existing production-style object. Dev must only ever read this server.
const objects = new Map([
  ["louvre-e2e/artworks/s3-reference/test.png", { body: readFileSync("e2e/fixtures/test.png"), contentType: "image/png" }],
]);
const port = Number(process.env.S3_MOCK_PORT ?? 19000);
let mutations = 0;

const server = createServer(async (request, response) => {
  const url = new URL(request.url, "http://localhost");
  if (url.pathname === "/health") {
    response.writeHead(200, { "content-type": "application/json" }).end(JSON.stringify({ mutations }));
    return;
  }
  if (request.method !== "GET") {
    mutations++;
    response.writeHead(405).end();
    return;
  }

  const [, bucket, ...keyParts] = url.pathname.split("/");
  const key = keyParts.join("/");
  if (!bucket || !key) {
    response.writeHead(404).end();
    return;
  }
  const storageKey = `${bucket}/${key}`;

  const object = objects.get(storageKey);
  if (!object) {
    response
      .writeHead(404, { "content-type": "application/xml" })
      .end("<Error><Code>NoSuchKey</Code><Message>Not found</Message></Error>");
    return;
  }
  response
    .writeHead(200, {
      "content-type": object.contentType,
      "content-length": object.body.length,
      etag: '"e2e"',
    })
    .end(object.body);
});

server.listen(port, "127.0.0.1");
process.on("SIGTERM", () => server.close(() => process.exit(0)));
