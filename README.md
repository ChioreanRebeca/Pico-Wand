
# [Pico Wand(Hand Movement Detector)](https://embedded-rust-101.wyliodrin.com/docs/project/2024/rebeca.chiorean#description)

## Description

Pico wand is useful device for physically impaired people. It uses a MPU6050 gyroscope that detects hand movements and interacts with the objects in the house. </br>
When pressing a push button it starts collecting the movement of your wand.</br>
It collects movements such as:</br>
  •	moving the wand and pressing the button raises the curtain</br>
  •	holding the wand still and pressing the button lowers the curtains</br>
All this information is then shown on a ST7789 display.</br>
This device could be used in relation to common use objects for people with disabilities.</br>
Example of usage: </br>
Let’s imagine someone that is bed bound and needs to adjust their smart curtains. They will pick up the Pico Wand, press the button and move the wand, then PicoW connected to an adjusted curtain will pick up the signal and move up the curtain. This sort of implementation could be done on other objects as well: a door, a light bulb and so on.</br>
My project is made in colaboration with my coleague Naomi Lita. [Link to her GitHub](https://github.com/UPB-FILS-MA/project-nimintz/blob/main/README.md)

## Connection
  • ST7789 LCD Display(240x240) has a SPI connection: <br />
    MISO: GP4 <br />
    MOSI: GP19 <br />
    CLK: GP18 <br />
    CS: GP17 <br />
    rst: GP0 <br />
    dc: GP16<br />
  • MPU6050 has a I2C connection:<br />
    SCL: GP21<br />
    SDA:GP20<br />
  • pushbutton with a pull up resistor on GP12<br />
<br />
About the KiCad schematic: My KiCad schematic uses the ST7735R display kicad_sym as a replacement for ST7789 display. Both have SPI connections and functionalities.

    
  

## Hardware

<!-- Fill out this table with all the hardware components that you mght need.

The format is 
```
| [Device](link://to/device) | This is used ... | [price](link://to/store) |

```

-->

| Device | Usage | Price |
|--------|--------|-------|
| [Rapspberry Pi Pico W](https://www.raspberrypi.com/documentation/microcontrollers/raspberry-pi-pico.html) | The microcontroller | [35 RON](https://www.optimusdigital.ro/en/raspberry-pi-boards/12394-raspberry-pi-pico-w.html) |
|[MPU6050 Accelerometer and Gyroscope Module](https://invensense.tdk.com/wp-content/uploads/2015/02/MPU-6000-Datasheet1.pdf)|Accelerometer sensor|[15,49 lei](https://www.optimusdigital.ro/en/inertial-sensors/96-mpu6050-accelerometer-and-gyroscope-module.html)|
|[LCD ST7789](https://www.rhydolabz.com/documents/33/ST7789.pdf)|Display|[149,00 lei](https://www.optimusdigital.ro/en/lcds/5345-adafruit-154-240x240-wide-angle-tft-lcd-display-with-microsd-st7789.html)|
|Breadboard|Prototyping|[9,98 lei](https://www.optimusdigital.ro/en/breadboards/8-breadboard-hq-830-points.html?search_query=bread+board&results=420)|
|Jumper Wires|Connecting components|[4,99 lei](https://www.optimusdigital.ro/en/wires-with-connectors/889-set-fire-tata-tata-10p-20-cm.html?search_query=jumper+wires&results=101)|
|Pushbutton switch 12mm|Signals the begining of the data collecting process|[1,99 lei](https://www.optimusdigital.ro/en/others/1118-blue-round-button-with-cover.html?search_query=button&results=510)|



## Links

<!-- Add a few links that got you the idea and that you think you will use for your project -->

1. [Hand Gesture Recognition for Numbers using TinyML](https://medium.com/@subirmaity/hand-gesture-recognition-for-numbers-using-tinyml-323d2a524c3e)
2. [Motion Recognition Using Raspberry Pi Pico](https://mjrobot.org/2021/03/12/tinyml-motion-recognition-using-raspberry-pi-pico/)
3. [Raspberry Pi Pico and Edge Impulse](https://www.hackster.io/shahizat/gesture-recognition-using-raspberry-pi-pico-and-edge-impulse-7a63b6)
4. [Using a screen display to output the movement](https://www.hackster.io/shubhamsantosh99/gesture-recognition-on-pico-using-edge-impulse-fd962e#overview)

