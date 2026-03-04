export type DriverType = 'modbus-rtu' | 'modbus-tcp' | 'opc-ua' | 'serial-raw' | 'mqtt';

export interface DriverConfig {
  id: string;
  name: string;
  type: DriverType;
  description?: string;
  settings: ModbusRtuSettings | ModbusTcpSettings | OpcUaSettings | SerialRawSettings | MqttSettings;
}

export interface ModbusRtuSettings {
  port: string;
  baudRate: number;
  dataBits: 7 | 8;
  parity: 'none' | 'even' | 'odd';
  stopBits: 1 | 2;
  slaveId: number;
}

export interface ModbusTcpSettings {
  host: string;
  port: number;
  unitId: number;
}

export interface OpcUaSettings {
  endpointUrl: string;
  securityMode: 'none' | 'sign' | 'sign-encrypt';
  username?: string;
  password?: string;
}

export interface SerialRawSettings {
  port: string;
  baudRate: number;
  dataBits: 7 | 8;
  parity: 'none' | 'even' | 'odd';
  stopBits: 1 | 2;
  lineEnding: 'none' | 'cr' | 'lf' | 'crlf';
}

export interface MqttSettings {
  broker: string;
  port: number;
  clientId: string;
  username?: string;
  password?: string;
  topic: string;
}

export const DRIVER_TYPE_LABELS: Record<DriverType, string> = {
  'modbus-rtu': 'Modbus RTU',
  'modbus-tcp': 'Modbus TCP',
  'opc-ua': 'OPC-UA',
  'serial-raw': 'Serial Raw',
  'mqtt': 'MQTT',
};

export function createDefaultDriverSettings(type: DriverType): DriverConfig['settings'] {
  switch (type) {
    case 'modbus-rtu':
      return {
        port: '/dev/ttyUSB0',
        baudRate: 9600,
        dataBits: 8,
        parity: 'none',
        stopBits: 1,
        slaveId: 1,
      };
    case 'modbus-tcp':
      return {
        host: '192.168.1.100',
        port: 502,
        unitId: 1,
      };
    case 'opc-ua':
      return {
        endpointUrl: 'opc.tcp://localhost:4840',
        securityMode: 'none',
      };
    case 'serial-raw':
      return {
        port: '/dev/ttyUSB0',
        baudRate: 115200,
        dataBits: 8,
        parity: 'none',
        stopBits: 1,
        lineEnding: 'crlf',
      };
    case 'mqtt':
      return {
        broker: 'localhost',
        port: 1883,
        clientId: 'senamby-client',
        topic: 'plant/data',
      };
  }
}
