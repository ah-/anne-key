Anne Pro Internals
==================

<a href="https://github.com/ah-/anne-key"><img src="images/ferris.png" width=30%/> <img src="images/anne.jpg" width=50%/></a>

This short book aims to document how the Anne Pro Keyboard really works internally as part of the [alternative firmware project](https://github.com/ah-/anne-key).

What is the Anne Pro and why should I care?
-------------------------------------------

It's a cheap (around $50 shipped on [AliExpress](https://www.aliexpress.com/item/Original-Techhunter-Anne-pro-Wireless-Bluetooth-Mechanical-Keyboard-with-RGB-Backlit-Gaming-Keyboard-61-Keys-Teclado/32821909053.html)) but good quality keyboard.

It's also very hackable, being built on two STM32L151 microcontrollers and with all programming pins neatly exposed. Open source support for STM32 chips is very good, they work particularly well with [embedded Rust](http://blog.japaric.io/quickstart/).

Contributing to this documentation
----------------------------------

Please help out documenting how the Anne Pro works to help others get started!

To do so edit [these Markdown files](https://github.com/ah-/anne-key/tree/master/docs) and send a Pull Request on GitHub.

You can preview your changes locally by building the docs with [gitbook](https://github.com/GitbookIO/gitbook).
Just run `gitbook serve` in the root folder and go to [http://localhost:4000/](http://localhost:4000/).
