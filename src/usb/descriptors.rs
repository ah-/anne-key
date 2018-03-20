pub const DEV_DESC: [u8; 18] = [
    0x12,        // bLength
    0x01,        // bDescriptorType (Device)
    0x00, 0x02,  // bcdUSB 2.00
    0x00,        // bDeviceClass (Use class information in the Interface Descriptors)
    0x00,        // bDeviceSubClass
    0x00,        // bDeviceProtocol
    0x40,        // bMaxPacketSize0 64
    0xFF, 0xFF,  // idVendor 0xFFFF
    0xFF, 0xFF,  // idProduct 0xFFFF
    0x01, 0x00,  // bcdDevice 0.01
    0x01,        // iManufacturer (String Index)
    0x02,        // iProduct (String Index)
    0x03,        // iSerialNumber (String Index)
    0x01,        // bNumConfigurations 1
];

pub const CONF_DESC: [u8; 34] = [
    0x09,        // bLength
    0x02,        // bDescriptorType (Configuration)
    0x22, 0x00,  // wTotalLength
    0x01,        // bNumInterfaces
    0x01,        // bConfigurationValue
    0x04,        // iConfiguration (String Index)
    0x80,        // bmAttributes
    0xFA,        // bMaxPower 500mA

    0x09,        // bLength
    0x04,        // bDescriptorType (Interface)
    0x00,        // bInterfaceNumber 0
    0x00,        // bAlternateSetting
    0x01,        // bNumEndpoints 1
    0x03,        // bInterfaceClass
    0x01,        // bInterfaceSubClass
    0x01,        // bInterfaceProtocol
    0x05,        // iInterface (String Index)

    0x09,        // bLength
    0x21,        // bDescriptorType (HID)
    0x11, 0x01,  // bcdHID 1.11
    0x00,        // bCountryCode
    0x01,        // bNumDescriptors
    0x22,        // bDescriptorType[0] (HID)
    0x3f, 0x00,  // wDescriptorLength[0] 63

    0x07,        // bLength
    0x05,        // bDescriptorType (Endpoint)
    0x81,        // bEndpointAddress (IN/D2H)
    0x03,        // bmAttributes (Interrupt)
    0x40, 0x00,  // wMaxPacketSize 64
    0x01,        // bInterval 1 (unit depends on device speed)
];

pub const HID_DESC: [u8; 9] = [
    0x09,        // bLength
    0x21,        // bDescriptorType (HID)
    0x11, 0x01,  // bcdHID 1.11
    0x00,        // bCountryCode
    0x01,        // bNumDescriptors
    0x22,        // bDescriptorType[0] (HID)
    0x3f, 0x00,  // wDescriptorLength[0] 63
];

pub const HID_REPORT_DESC: [u8; 63] = [
    0x05, 0x01,        // Usage Page (Generic Desktop Ctrls)
    0x09, 0x06,        // Usage (Keyboard)
    0xA1, 0x01,        // Collection (Application)
    0x05, 0x07,        //   Usage Page (Kbrd/Keypad)
    0x19, 0xE0,        //   Usage Minimum (0xE0)
    0x29, 0xE7,        //   Usage Maximum (0xE7)
    0x15, 0x00,        //   Logical Minimum (0)
    0x25, 0x01,        //   Logical Maximum (1)
    0x75, 0x01,        //   Report Size (1)
    0x95, 0x08,        //   Report Count (8)
    0x81, 0x02,        //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x95, 0x01,        //   Report Count (1)
    0x75, 0x08,        //   Report Size (8)
    0x81, 0x01,        //   Input (Const,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x95, 0x05,        //   Report Count (5)
    0x75, 0x01,        //   Report Size (1)
    0x05, 0x08,        //   Usage Page (LEDs)
    0x19, 0x01,        //   Usage Minimum (Num Lock)
    0x29, 0x05,        //   Usage Maximum (Kana)
    0x91, 0x02,        //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x95, 0x01,        //   Report Count (1)
    0x75, 0x03,        //   Report Size (3)
    0x91, 0x01,        //   Output (Const,Array,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x95, 0x06,        //   Report Count (6)
    0x75, 0x08,        //   Report Size (8)
    0x15, 0x00,        //   Logical Minimum (0)
    0x25, 0x65,        //   Logical Maximum (101)
    0x05, 0x07,        //   Usage Page (Kbrd/Keypad)
    0x19, 0x00,        //   Usage Minimum (0x00)
    0x29, 0x65,        //   Usage Maximum (0x65)
    0x81, 0x00,        //   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xC0,              // End Collection
];

pub const DEVICE_QUALIFIER: [u8; 10] = [
    0x0A,        // bLength
    0x06,        // bDescriptorType (Device Qualifier)
    0x00, 0x02,  // bcdUSB 2.00
    0x00,        // bDeviceClass (Use class information in the Interface Descriptors)
    0x00,        // bDeviceSubClass
    0x40,        // bDeviceProtocol 
    0x01,        // bMaxPacketSize0 1
    0x00,        // bNumConfigurations 0
    0x00,        // bReserved
];

pub const LANG_STR: [u8; 4] = [
    0x04, 0x03, //
    0x09, 0x04, // English - US
];

pub const MANUFACTURER_STR: [u8; 38] = [
    0x26, 0x03, //
    0x52, 0x00, // R
    0x75, 0x00, // u
    0x73, 0x00, // s
    0x74, 0x00, // t
    0x79, 0x00, // y
    0x20, 0x00, //  
    0x4d, 0x00, // M
    0x61, 0x00, // a
    0x6e, 0x00, // n
    0x75, 0x00, // u
    0x66, 0x00, // f
    0x61, 0x00, // a
    0x63, 0x00, // c
    0x74, 0x00, // t
    0x75, 0x00, // u
    0x72, 0x00, // r
    0x65, 0x00, // e
    0x72, 0x00, // r
];

pub const PRODUCT_STR: [u8; 28] = [
    0x1c, 0x03, //
    0x52, 0x00, // R
    0x75, 0x00, // u
    0x73, 0x00, // s
    0x74, 0x00, // t
    0x79, 0x00, // y
    0x20, 0x00, //  
    0x50, 0x00, // P
    0x72, 0x00, // r
    0x6f, 0x00, // o
    0x64, 0x00, // d
    0x75, 0x00, // u
    0x63, 0x00, // c
    0x74, 0x00, // t
];

pub const SERIAL_NUMBER_STR: [u8; 14] = [
    0x0e, 0x03, //
    0x31, 0x00, // 1
    0x32, 0x00, // 2
    0x33, 0x00, // 3
    0x41, 0x00, // A
    0x42, 0x00, // B
    0x43, 0x00, // C
];

pub const CONF_STR: [u8; 40] = [
    0x28, 0x03, //
    0x52, 0x00, // R
    0x75, 0x00, // u
    0x73, 0x00, // s
    0x74, 0x00, // t
    0x79, 0x00, // y
    0x20, 0x00, //
    0x43, 0x00, // C
    0x6f, 0x00, // o
    0x6e, 0x00, // n
    0x66, 0x00, // f
    0x69, 0x00, // i
    0x67, 0x00, // g
    0x75, 0x00, // u
    0x72, 0x00, // r
    0x61, 0x00, // a
    0x74, 0x00, // t
    0x69, 0x00, // i
    0x6f, 0x00, // o
    0x6e, 0x00, // n
];

pub const INTERFACE_STR: [u8; 32] = [
    0x20, 0x03, //
    0x52, 0x00, // R
    0x75, 0x00, // u
    0x73, 0x00, // s
    0x74, 0x00, // t
    0x79, 0x00, // y
    0x20, 0x00, //
    0x49, 0x00, // I
    0x6e, 0x00, // n
    0x74, 0x00, // t
    0x65, 0x00, // e
    0x72, 0x00, // r
    0x66, 0x00, // f
    0x61, 0x00, // a
    0x63, 0x00, // c
    0x65, 0x00, // e
];
