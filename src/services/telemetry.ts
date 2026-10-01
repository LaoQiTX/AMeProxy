/** Owns one socket and one retry timer; stop() invalidates every old callback. */
export class TelemetrySocket {
  private socket: WebSocket | null = null;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private stopped = false;
  private retry = 1000;

  constructor(private url: string, private receive: (data: any) => void) {
    this.connect();
  }

  private connect() {
    if (this.stopped) return;
    try {
      const socket = new WebSocket(this.url);
      this.socket = socket;
      socket.onopen = () => { if (this.socket === socket) this.retry = 1000; };
      socket.onmessage = event => {
        if (this.stopped || this.socket !== socket) return;
        try { this.receive(JSON.parse(event.data)); } catch { /* Ignore malformed frames. */ }
      };
      socket.onerror = () => socket.close();
      socket.onclose = () => {
        if (this.socket !== socket || this.stopped) return;
        this.socket = null;
        this.schedule();
      };
    } catch { this.schedule(); }
  }

  private schedule() {
    if (this.stopped || this.timer) return;
    this.timer = setTimeout(() => { this.timer = null; this.connect(); }, this.retry);
    this.retry = Math.min(this.retry * 2, 10000);
  }

  stop() {
    this.stopped = true;
    if (this.timer) clearTimeout(this.timer);
    this.timer = null;
    const socket = this.socket;
    this.socket = null;
    socket?.close();
  }
}

export async function runLimited<T>(items: T[], limit: number, task: (item: T) => Promise<void>) {
  let next = 0;
  await Promise.all(Array.from({ length: Math.min(limit, items.length) }, async () => {
    while (next < items.length) await task(items[next++]);
  }));
}
