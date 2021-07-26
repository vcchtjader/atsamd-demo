
/**
 * \file
 *
 * \brief RTC related functionality implementation.
 *
 * Copyright (c) 2017 Microchip Technology Inc. and its subsidiaries.
 *
 * \asf_license_start
 *
 * \page License
 *
 * Subject to your compliance with these terms, you may use Microchip
 * software and any derivatives exclusively with Microchip products.
 * It is your responsibility to comply with third party license terms applicable
 * to your use of third party software (including open source software) that
 * may accompany Microchip software.
 *
 * THIS SOFTWARE IS SUPPLIED BY MICROCHIP "AS IS". NO WARRANTIES,
 * WHETHER EXPRESS, IMPLIED OR STATUTORY, APPLY TO THIS SOFTWARE,
 * INCLUDING ANY IMPLIED WARRANTIES OF NON-INFRINGEMENT, MERCHANTABILITY,
 * AND FITNESS FOR A PARTICULAR PURPOSE. IN NO EVENT WILL MICROCHIP BE
 * LIABLE FOR ANY INDIRECT, SPECIAL, PUNITIVE, INCIDENTAL OR CONSEQUENTIAL
 * LOSS, DAMAGE, COST OR EXPENSE OF ANY KIND WHATSOEVER RELATED TO THE
 * SOFTWARE, HOWEVER CAUSED, EVEN IF MICROCHIP HAS BEEN ADVISED OF THE
 * POSSIBILITY OR THE DAMAGES ARE FORESEEABLE.  TO THE FULLEST EXTENT
 * ALLOWED BY LAW, MICROCHIP'S TOTAL LIABILITY ON ALL CLAIMS IN ANY WAY
 * RELATED TO THIS SOFTWARE WILL NOT EXCEED THE AMOUNT OF FEES, IF ANY,
 * THAT YOU HAVE PAID DIRECTLY TO MICROCHIP FOR THIS SOFTWARE.
 *
 * \asf_license_stop
 *
 */

#include "rtc_lite.h"

/**
 * \brief Initialize RTC interface
 */
int8_t CALENDAR_0_init()
{

	hri_rtcmode2_wait_for_sync(RTC, RTC_MODE2_SYNCBUSY_SWRST);
	if (hri_rtcmode2_get_CTRLA_ENABLE_bit(RTC)) {
		hri_rtcmode2_clear_CTRLA_ENABLE_bit(RTC);
		hri_rtcmode2_wait_for_sync(RTC, RTC_MODE2_SYNCBUSY_ENABLE);
	}
	hri_rtcmode2_set_CTRLA_SWRST_bit(RTC);
	hri_rtcmode2_wait_for_sync(RTC, RTC_MODE2_SYNCBUSY_SWRST);

	hri_rtcmode2_write_CTRLA_reg(
	    RTC,
	    1 << RTC_MODE2_CTRLA_CLOCKSYNC_Pos        /* CLOCK Read Synchronization Enable: enabled */
	        | 11 << RTC_MODE2_CTRLA_PRESCALER_Pos /* Setting: 11 */
	        | 0 << RTC_MODE2_CTRLA_MATCHCLR_Pos   /* Clear on Match: disabled */
	        | 0 << RTC_MODE2_CTRLA_CLKREP_Pos     /* Clock Representation: disabled */
	        | 0x2 << RTC_MODE2_CTRLA_MODE_Pos     /* Operating Mode: 0x2 */
	        | 0 << RTC_MODE2_CTRLA_GPTRST_Pos     /* GP Registers Reset On Tamper Enable: disabled */
	        | 0 << RTC_MODE2_CTRLA_BKTRST_Pos);   /* GP Registers Reset On Tamper Enable: disabled */

	// hri_rtcmode2_write_CLOCK_reg(RTC,0x0 << RTC_MODE2_CLOCK_YEAR_Pos /* Year: 0x0 */
	//		 | 0x0 << RTC_MODE2_CLOCK_MONTH_Pos /* Month: 0x0 */
	//		 | 0x0 << RTC_MODE2_CLOCK_DAY_Pos /* Day: 0x0 */
	//		 | 0x0 << RTC_MODE2_CLOCK_HOUR_Pos /* Hour: 0x0 */
	//		 | 0x0 << RTC_MODE2_CLOCK_MINUTE_Pos /* Minute: 0x0 */
	//		 | 0x0 << RTC_MODE2_CLOCK_SECOND_Pos); /* Second: 0x0 */

	// hri_rtcmode2_write_EVCTRL_reg(RTC,0 << RTC_MODE2_EVCTRL_OVFEO_Pos /* Overflow Event Output Enable: disabled */
	//		 | 0 << RTC_MODE2_EVCTRL_TAMPEVEI_Pos /* Tamper Event Input Enable: disabled */
	//		 | 0 << RTC_MODE2_EVCTRL_TAMPEREO_Pos /* Tamper Event output Enable: disabled */
	//		 | 0 << RTC_MODE2_EVCTRL_ALARMEO0_Pos /* Alarmo 0 Event Output Enable: disabled */
	//		 | 0 << RTC_MODE2_EVCTRL_ALARMEO1_Pos /* Alarmo 1 Event Output Enable: disabled */
	//		 | 0 << RTC_MODE2_EVCTRL_PEREO0_Pos /* Periodic Interval 0 Event Output Enable: disabled */
	//		 | 0 << RTC_MODE2_EVCTRL_PEREO1_Pos /* Periodic Interval 1 Event Output Enable: disabled */
	//		 | 0 << RTC_MODE2_EVCTRL_PEREO2_Pos /* Periodic Interval 2 Event Output Enable: disabled */
	//		 | 0 << RTC_MODE2_EVCTRL_PEREO3_Pos /* Periodic Interval 3 Event Output Enable: disabled */
	//		 | 0 << RTC_MODE2_EVCTRL_PEREO4_Pos /* Periodic Interval 4 Event Output Enable: disabled */
	//		 | 0 << RTC_MODE2_EVCTRL_PEREO5_Pos /* Periodic Interval 5 Event Output Enable: disabled */
	//		 | 0 << RTC_MODE2_EVCTRL_PEREO6_Pos /* Periodic Interval 6 Event Output Enable: disabled */
	//		 | 0 << RTC_MODE2_EVCTRL_PEREO7_Pos); /* Periodic Interval 7 Event Output Enable: disabled */

	hri_rtcmode2_write_INTEN_reg(RTC,
	                             0 << RTC_MODE2_INTENSET_OVF_Pos          /* Overflow Interrupt enable: disabled */
	                                 | 1 << RTC_MODE2_INTENSET_TAMPER_Pos /* Tamper Interrupt Enable: enabled */
	                                 | 0 << RTC_MODE2_INTENSET_ALARM0_Pos /* Alarm 0 Interrupt Enable: disabled */
	                                 | 0 << RTC_MODE2_INTENSET_ALARM1_Pos /* Alarm 1 Interrupt Enable: disabled */
	                                 | 0 << RTC_MODE2_INTENSET_PER0_Pos   /* Setting: disabled */
	                                 | 0 << RTC_MODE2_INTENSET_PER1_Pos   /* Setting: disabled */
	                                 | 0 << RTC_MODE2_INTENSET_PER2_Pos   /* Setting: disabled */
	                                 | 0 << RTC_MODE2_INTENSET_PER3_Pos   /* Setting: disabled */
	                                 | 0 << RTC_MODE2_INTENSET_PER4_Pos   /* Setting: disabled */
	                                 | 0 << RTC_MODE2_INTENSET_PER5_Pos   /* Setting: disabled */
	                                 | 0 << RTC_MODE2_INTENSET_PER6_Pos   /* Setting: disabled */
	                                 | 0 << RTC_MODE2_INTENSET_PER7_Pos); /* Setting: disabled */

	// hri_rtcmode2_write_ALARM_reg(RTC, 0 ,0x0 << RTC_MODE2_ALARM_YEAR_Pos /* Year0: 0x0 */
	//		 | 0x0 << RTC_MODE2_ALARM_MONTH_Pos /* Month0: 0x0 */
	//		 | 0x0 << RTC_MODE2_ALARM_DAY_Pos /* Day0: 0x0 */
	//		 | 0x0 << RTC_MODE2_ALARM_HOUR_Pos /* Hour0: 0x0 */
	//		 | 0x0 << RTC_MODE2_ALARM_MINUTE_Pos /* Minute0: 0x0 */
	//		 | 0x0 << RTC_MODE2_ALARM_SECOND_Pos); /* Second0: 0x0 */

	// hri_rtcmode2_write_MASK_reg(RTC, 0 ,0); /* Setting: 0 */

	// hri_rtcmode2_write_DBGCTRL_reg(RTC,0); /* Run in debug: 0 */

	// hri_rtcmode2_write_FREQCORR_reg(RTC,0 << RTC_FREQCORR_SIGN_Pos /* Correction Sign: disabled */
	//		 | 0x0); /* Correction Value: 0x0 */

	// hri_rtcmode2_write_GP_reg(RTC, 0 ,0x0); /* General Purpose: 0x0 */

	// hri_rtcmode2_write_GP_reg(RTC, 1 ,0x0); /* General Purpose: 0x0 */

	// hri_rtcmode2_write_CTRLB_reg(RTC,0 << RTC_MODE2_CTRLB_ACTF_Pos /* Setting: 0 */
	//		 | 0 << RTC_MODE2_CTRLB_DEBF_Pos /* Setting: 0 */
	//		 | 0 << RTC_MODE2_CTRLB_DMAEN_Pos /* DMA Enable: disabled */
	//		 | 0 << RTC_MODE2_CTRLB_RTCOUT_Pos /* RTC Output Enable: disabled */
	//		 | 0 << RTC_MODE2_CTRLB_DEBASYNC_Pos /* Debouncer Asynchronous Enable: disabled */
	//		 | 0 << RTC_MODE2_CTRLB_DEBMAJ_Pos /* Debouncer Majority Enable: disabled */
	//		 | 0 << RTC_MODE2_CTRLB_GP2EN_Pos /* General Purpose 2 Enable: disabled */
	//		 | 0 << RTC_MODE2_CTRLB_GP0EN_Pos); /* General Purpose 0 Enable: disabled */

	hri_rtcmode2_write_TAMPCTRL_reg(RTC,
	                                1 << RTC_TAMPCTRL_DEBNC0_Pos         /* Debounce Enable 0: enabled */
	                                    | 1 << RTC_TAMPCTRL_DEBNC1_Pos   /* Debounce Enable 1: enabled */
	                                    | 1 << RTC_TAMPCTRL_DEBNC2_Pos   /* Debounce Enable 2: enabled */
	                                    | 1 << RTC_TAMPCTRL_DEBNC3_Pos   /* Debounce Enable 3: enabled */
	                                    | 1 << RTC_TAMPCTRL_DEBNC4_Pos   /* Debounce Enable 4: enabled */
	                                    | 0 << RTC_TAMPCTRL_TAMLVL0_Pos  /* Tamper Level Select 0: disabled */
	                                    | 0 << RTC_TAMPCTRL_TAMLVL1_Pos  /* Tamper Level Select 1: disabled */
	                                    | 0 << RTC_TAMPCTRL_TAMLVL2_Pos  /* Tamper Level Select 2: disabled */
	                                    | 0 << RTC_TAMPCTRL_TAMLVL3_Pos  /* Tamper Level Select 3: disabled */
	                                    | 0 << RTC_TAMPCTRL_TAMLVL4_Pos  /* Tamper Level Select 4: disabled */
	                                    | 1 << RTC_TAMPCTRL_IN0ACT_Pos   /* Tamper Input 0 Action: 1 */
	                                    | 1 << RTC_TAMPCTRL_IN1ACT_Pos   /* Tamper Input 1 Action: 1 */
	                                    | 1 << RTC_TAMPCTRL_IN2ACT_Pos   /* Tamper Input 2 Action: 1 */
	                                    | 1 << RTC_TAMPCTRL_IN3ACT_Pos   /* Tamper Input 3 Action: 1 */
	                                    | 1 << RTC_TAMPCTRL_IN4ACT_Pos); /* Tamper Input 4 Action: 1 */

	// hri_rtcmode2_write_TAMPID_reg(RTC,0 << RTC_TAMPID_TAMPEVT_Pos /* Tamper Event Detected: disabled */
	//		 | 0 << RTC_TAMPID_TAMPID0_Pos /* Tamper Input 0 Detected: disabled */
	//		 | 0 << RTC_TAMPID_TAMPID1_Pos /* Tamper Input 1 Detected: disabled */
	//		 | 0 << RTC_TAMPID_TAMPID2_Pos /* Tamper Input 2 Detected: disabled */
	//		 | 0 << RTC_TAMPID_TAMPID3_Pos /* Tamper Input 3 Detected: disabled */
	//		 | 0 << RTC_TAMPID_TAMPID4_Pos); /* Tamper Input 4 Detected: disabled */

	// hri_rtcmode2_write_MASK_reg(RTC, 1 ,0); /* Setting: 0 */

	// hri_rtcmode2_write_ALARM_reg(RTC, 1 ,0x0 << RTC_MODE2_ALARM_YEAR_Pos /* Year1: 0x0 */
	//		 | 0x0 << RTC_MODE2_ALARM_MONTH_Pos /* Month1: 0x0 */
	//		 | 0x0 << RTC_MODE2_ALARM_DAY_Pos /* Day1: 0x0 */
	//		 | 0x0 << RTC_MODE2_ALARM_HOUR_Pos /* Hour1: 0x0 */
	//		 | 0x0 << RTC_MODE2_ALARM_MINUTE_Pos /* Minute1: 0x0 */
	//		 | 0x0 << RTC_MODE2_ALARM_SECOND_Pos); /* Second1: 0x0 */

	// hri_rtcmode2_write_GP_reg(RTC, 2 ,0x0); /* General Purpose: 0x0 */

	// hri_rtcmode2_write_GP_reg(RTC, 3 ,0x0); /* General Purpose: 0x0 */

	// hri_rtcmode2_write_BKUP_reg(RTC, 0 ,0x0); /* Backup0: 0x0 */

	// hri_rtcmode2_write_BKUP_reg(RTC, 1 ,0x0); /* Backup1: 0x0 */

	// hri_rtcmode2_write_BKUP_reg(RTC, 2 ,0x0); /* Backup2: 0x0 */

	// hri_rtcmode2_write_BKUP_reg(RTC, 3 ,0x0); /* Backup3: 0x0 */

	// hri_rtcmode2_write_BKUP_reg(RTC, 4 ,0x0); /* Backup4: 0x0 */

	// hri_rtcmode2_write_BKUP_reg(RTC, 5 ,0x0); /* Backup5: 0x0 */

	// hri_rtcmode2_write_BKUP_reg(RTC, 6 ,0x0); /* Backup6: 0x0 */

	// hri_rtcmode2_write_BKUP_reg(RTC, 7 ,0x0); /* Backup7: 0x0 */

	hri_rtcmode2_write_CTRLA_ENABLE_bit(RTC, 1 << RTC_MODE2_CTRLA_ENABLE_Pos); /* Enable: enabled */

	return 0;
}
