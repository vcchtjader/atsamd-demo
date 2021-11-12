/*
 * Code generated from Atmel Start.
 *
 * This file will be overwritten when reconfiguring your Atmel Start project.
 * Please copy examples or other code you want to keep to a separate file
 * to avoid losing it when reconfiguring.
 */
#ifndef DRIVER_INIT_INCLUDED
#define DRIVER_INIT_INCLUDED

#include "atmel_start_pins.h"

#ifdef __cplusplus
extern "C" {
#endif

#include <hal_atomic.h>
#include <hal_delay.h>
#include <hal_gpio.h>
#include <hal_init.h>
#include <hal_io.h>
#include <hal_sleep.h>

#include <hal_adc_sync.h>

#include <hal_adc_sync.h>
#include <hal_aes_sync.h>

#include <hal_crc_sync.h>

#include <hal_sha_sync.h>

#include <hal_flash.h>

#include <string.h>
#include "CryptoLib_typedef_pb.h"
#include "CryptoLib_mapping_pb.h"
#include "CryptoLib_cf_pb.h"
#include "CryptoLib_Headers_pb.h"

#include <rtc_lite.h>
#include <hal_usart_async.h>
#include <hal_usart_async.h>
#include <hal_usart_async.h>
#include <hal_spi_m_sync.h>
#include <hal_usart_async.h>
#include <hal_usart_async.h>
#include <hal_spi_m_sync.h>

#include <hal_i2c_m_sync.h>
#include <hal_timer.h>
#include <hpl_tc_base.h>
#include <hal_pwm.h>
#include <hpl_tc_base.h>

#include <hal_wdt.h>
#include <hal_can_async.h>
#include <hal_can_async.h>

extern struct adc_sync_descriptor ADC_0;

extern struct adc_sync_descriptor ADC_1;
extern struct aes_sync_descriptor CRYPTOGRAPHY_1;
extern struct crc_sync_descriptor CRC_0;
extern struct sha_sync_descriptor HASH_ALGORITHM_0;

extern struct flash_descriptor       FLASH_0;
extern PPUKCL_PARAM                  pvPUKCLParam;
extern PUKCL_PARAM                   PUKCLParam;
extern struct usart_async_descriptor USART_2;
extern struct usart_async_descriptor USART_1;
extern struct usart_async_descriptor USART_0;
extern struct spi_m_sync_descriptor  SPI_0;
extern struct usart_async_descriptor USART_5;
extern struct usart_async_descriptor USART_3;
extern struct spi_m_sync_descriptor  SPI_1;

extern struct i2c_m_sync_desc  I2C_0;
extern struct timer_descriptor TIMER_1;

extern struct pwm_descriptor PWM_0;

extern struct wdt_descriptor       WDT_0;
extern struct can_async_descriptor CAN_1;
extern struct can_async_descriptor CAN_0;

void ADC_0_PORT_init(void);
void ADC_0_CLOCK_init(void);
void ADC_0_init(void);

void ADC_1_PORT_init(void);
void ADC_1_CLOCK_init(void);
void ADC_1_init(void);

void FLASH_0_init(void);
void FLASH_0_CLOCK_init(void);

void   CALENDAR_0_CLOCK_init(void);
int8_t CALENDAR_0_init(void);

void USART_2_PORT_init(void);
void USART_2_CLOCK_init(void);
void USART_2_init(void);

void USART_1_PORT_init(void);
void USART_1_CLOCK_init(void);
void USART_1_init(void);

void USART_0_PORT_init(void);
void USART_0_CLOCK_init(void);
void USART_0_init(void);

void SPI_0_PORT_init(void);
void SPI_0_CLOCK_init(void);
void SPI_0_init(void);

void USART_5_PORT_init(void);
void USART_5_CLOCK_init(void);
void USART_5_init(void);

void USART_3_PORT_init(void);
void USART_3_CLOCK_init(void);
void USART_3_init(void);

void SPI_1_PORT_init(void);
void SPI_1_CLOCK_init(void);
void SPI_1_init(void);

void I2C_0_CLOCK_init(void);
void I2C_0_init(void);
void I2C_0_PORT_init(void);

void PWM_0_PORT_init(void);
void PWM_0_CLOCK_init(void);
void PWM_0_init(void);

void WDT_0_CLOCK_init(void);
void WDT_0_init(void);

/**
 * \brief Perform system initialization, initialize pins and clocks for
 * peripherals
 */
void system_init(void);

#ifdef __cplusplus
}
#endif
#endif // DRIVER_INIT_INCLUDED
