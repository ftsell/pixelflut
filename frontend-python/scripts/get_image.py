#!/usr/bin/env python3

from PIL import Image
import argparse
import time
import os
import sys

SRC_DIR = os.path.abspath(os.path.join(os.path.dirname(os.path.dirname(__file__)), "src"))
sys.path.append(SRC_DIR)

from pixelflut_client import Client, BinaryAlgorithms


def parse_args():
    parser = argparse.ArgumentParser()

    parser.add_argument("server_hostname")
    parser.add_argument("server_port")

    return parser.parse_args()


if __name__ == "__main__":
    args = parse_args()

    client = Client()
    client.connect(args.server_hostname, int(args.server_port))
    im = Image.new("RGB", (client.size[0], client.size[1]))

    print("Receiving canvas", end="")
    start = time.time()
    pixels = client.receive_binary(BinaryAlgorithms.RgbBase64)
    end = time.time()
    print(f"     [{end - start}s]")

    print("Painting local canvas", end="")
    start = time.time()
    for ix in range(0, client.size[0]):
        for iy in range(0, client.size[1]):
            pixel_index = (ix * client.size[1] + iy) * 3

            r = pixels[pixel_index]
            g = pixels[pixel_index + 1]
            b = pixels[pixel_index + 2]

            im.putpixel((ix, iy), (r, g, b))
    end = time.time()
    print(f"    [{end - start}s]")

    im.show()
