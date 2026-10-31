import { createServer } from "node:http";

const objects = new Map();
const port = Number(process.env.S3_MOCK_PORT ?? 19000);

const server = createServer(async (request, response) => {
  const url = new URL(request.url, "http://localhost");
  if (url.pathname === "/health") {
    response.writeHead(200).end("ok");
    return;
  }

  const [, bucket, ...keyParts] = url.pathname.split("/");
  const key = keyParts.join("/");
  if (!bucket || !key) {
    response.writeHead(404).end();
    return;
  }
  const storageKey = `${bucket}/${key}`;

  if (request.method === "PUT") {
    const chunks = [];
    for await (const chunk of request) chunks.push(chunk);
    objects.set(storageKey, {
      body: Buffer.concat(chunks),
      contentType: request.headers["content-type"] ?? "application/octet-stream",
    });
    response.writeHead(200, { etag: '"e2e"' }).end();
    return;
  }

  if (request.method === "GET") {
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
    return;
  }

  if (request.method === "DELETE") {
    objects.delete(storageKey);
    response.writeHead(204).end();
    return;
  }

  response.writeHead(405).end();
});

server.listen(port, "127.0.0.1");
process.on("SIGTERM", () => server.close(() => process.exit(0)));
