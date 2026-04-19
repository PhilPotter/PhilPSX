// SPDX-License-Identifier: GPL-3.0
// bridges.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

// All the logic for bridging the different components and allowing them
// to pass references down a call stack to each other belongs inside this
// module.

/// This module contains CPU bridging functionality.
pub mod cpu;

/// This module contains motherboard bridging functionality.
pub mod motherboard;