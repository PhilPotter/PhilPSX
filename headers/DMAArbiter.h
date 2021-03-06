/*
 * This header file provides the public API for the class of PhilPSX that
 * handles the DMA transfers of the PlayStation hardware.
 * 
 * DMAArbiter.h - Copyright Phillip Potter, 2020, under GPLv3
 */
#ifndef PHILPSX_DMAARBITER_HEADER
#define PHILPSX_DMAARBITER_HEADER

// System includes
#include <stdint.h>

// Typedefs
typedef struct DMAArbiter DMAArbiter;

// Includes
#include "CDROMDrive.h"
#include "R3051.h"
#include "GPU.h"
#include "SystemInterlink.h"

// Public functions
DMAArbiter *construct_DMAArbiter(void);
void destruct_DMAArbiter(DMAArbiter *dma);
int8_t DMAArbiter_readByte(DMAArbiter *dma, int32_t address);
int32_t DMAArbiter_readWord(DMAArbiter *dma, int32_t address);
void DMAArbiter_setCdrom(DMAArbiter *dma, CDROMDrive *cdrom);
void DMAArbiter_setCpu(DMAArbiter *dma, R3051 *cpu);
void DMAArbiter_setGpu(DMAArbiter *dma, GPU *gpu);
void DMAArbiter_setMemoryInterface(DMAArbiter *dma, SystemInterlink *smi);
void DMAArbiter_writeByte(DMAArbiter *dma, int32_t address, int8_t value);
void DMAArbiter_writeWord(DMAArbiter *dma, int32_t address, int32_t word);

#endif