export interface Connection {
  id: number;
  host: string;
  ip: string;
  process: string;
  rule: string;
  group: string;
  speed: string;
  time: string;
}

export interface Rule {
  type: string;
  payload: string;
  strategy: string;
}

export interface Log {
  time: string;
  level: string;
  msg: string;
}

export interface ProxyGroup {
  name: string;
  type: string;
  selected: string;
  options: string[];
}

export interface Subscription {
  name: string;
  url: string;
  count: number;
  updateTime: string;
}

export interface Proxy {
  name: string;
  type: string;
  delay: number;
  region: string;
}

export interface Theme {
  id: string;
  name: string;
  color: string;
  bg: string;
  text: string;
  btn: string;
  shadow: string;
}

export interface TrafficData {
  up: string;
  down: string;
}
