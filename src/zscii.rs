// See 3.5.3 @ https://www.inform-fiction.org/zmachine/standards/z1point1/sect03.html
const ZSCII_MAP: [[char; 32]; 3] = [
    [
        ' ', '\0', '\0', '\0', '\0', '\0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
        'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ],
    [
        '\0', '\0', '\0', '\0', '\0', '\0', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
        'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ],
    [
        '\0', '\0', '\0', '\0', '\0', '\0', ' ', '\n', '0', '1', '2', '3', '4', '5', '6', '7', '8',
        '9', '.', ',', '!', '?', '_', '#', '\'', '"', '/', '\\', '-', ':', '(', ')',
    ],
];
