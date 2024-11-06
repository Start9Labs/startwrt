# Identity PSK

- `opkg remove wpad-basic-mbedtls`
- `opkg install wpad-wolfssl`
- `opkg install hostapd-utils`

## /etc/config/wireless
- under `wifi-iface`
```
        option encryption 'psk2'
        option wpa_psk_file '/etc/hostapd.wpa_psk'
```

## /etc/hostapd.wpa_psk (example)
```
00:00:00:00:00:00 password
keyid=guest 00:00:00:00:00:00 guestpassword
```

## check key of client
- `hostapd_cli all_sta`
