import socket
import tkinter as tk
import logging

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('ball_plate_emulator.log'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)

UDP_ADDR = ("127.0.0.1", 5007)
UPDATE_INTERVAL = 5 # ms

try:
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    logger.info("Socket UDP inicializado com sucesso")
except Exception as e:
    logger.error(f"Erro ao inicializar o socket: {e}")
    raise

class BallPlateEmulator(tk.Canvas):
    def __init__(self, root, width=800, height=700, ball_radius=15):
        super().__init__(root, width=width, height=height, bg="white")
        self.pack()
        self.width = width
        self.height = height
        self.ball_radius = ball_radius

        self.ball_x = width / 2
        self.ball_y = height / 2

        r = ball_radius
        self.ball = self.create_oval(
            self.ball_x - r, self.ball_y - r,
            self.ball_x + r, self.ball_y + r,
            fill="blue"
        )

        self.bind("<B1-Motion>", self.on_drag)
        self.after(UPDATE_INTERVAL, self.send_position)

        logger.info(f"Emulador inicializado: width={width}, height={height}, ball_radius={ball_radius}")

    def on_drag(self, event):
        try:
            x = min(max(event.x, self.ball_radius), self.width - self.ball_radius)
            y = min(max(event.y, self.ball_radius), self.height - self.ball_radius)
            self.ball_x, self.ball_y = x, y
            self.coords(
                self.ball,
                x - self.ball_radius, y - self.ball_radius,
                x + self.ball_radius, y + self.ball_radius
            )
            logger.debug(f"Bola movida para posição: x={x}, y={y}")
        except Exception as e:
            logger.error(f"Erro ao mover a bola: {e}")

    def send_position(self):
        try:
            msg = f"{float(self.ball_x):.2f},{float(self.ball_y):.2f}".encode()
            sock.sendto(msg, UDP_ADDR)
            logger.info(f"Posição enviada via UDP: x={self.ball_x:.2f}, y={self.ball_y:.2f}")
        except Exception as e:
            logger.error(f"Erro ao enviar posição via UDP: {e}")
        self.after(UPDATE_INTERVAL, self.send_position)

if __name__ == "__main__":
    try:
        root = tk.Tk()
        root.title("Emulador Ball and Plate")
        logger.info("Aplicação Tkinter iniciada")
        BallPlateEmulator(root)
        root.mainloop()
    except Exception as e:
        logger.error(f"Erro na execução da aplicação: {e}")
        raise