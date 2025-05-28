import socket
import tkinter as tk

UDP_ADDR        = ("127.0.0.1", 5007)
UPDATE_INTERVAL = 5 #ms

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

class BallPlateEmulator(tk.Canvas):
    def __init__(self, root, width=800, height=700, ball_radius=15):
        super().__init__(root, width=width, height=height, bg="white")
        self.pack()
        self.width  = width
        self.height = height
        self.ball_radius = ball_radius

        self.ball_x = width  / 2
        self.ball_y = height / 2

        r = ball_radius
        self.ball = self.create_oval(
            self.ball_x - r, self.ball_y - r,
            self.ball_x + r, self.ball_y + r,
            fill="blue"
        )

        self.bind("<B1-Motion>", self.on_drag)

        self.after(UPDATE_INTERVAL, self.send_position)

    def on_drag(self, event):
        x = min(max(event.x, self.ball_radius), self.width  - self.ball_radius)
        y = min(max(event.y, self.ball_radius), self.height - self.ball_radius)
        self.ball_x, self.ball_y = x, y
        self.coords(
            self.ball,
            x - self.ball_radius, y - self.ball_radius,
            x + self.ball_radius, y + self.ball_radius
        )

    def send_position(self):
        msg = f"{float(self.ball_x)},{float(self.ball_y)}".encode()
        sock.sendto(msg, UDP_ADDR)
        self.after(UPDATE_INTERVAL, self.send_position)


if __name__ == "__main__":
    root = tk.Tk()
    root.title("Emulador Ball and Plate")
    BallPlateEmulator(root)
    root.mainloop()
