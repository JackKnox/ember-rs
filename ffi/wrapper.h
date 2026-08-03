// Ember C ABI wrapper header.
// This header is the input to the rust bindgen library, everthing included
// into this file will be processed by bindgen and inserted into the ffi crate.
//

// Core domain
#include <ember/core/format.h>
#include <ember/core/logger.h>
#include <ember/core/math_types.h>
#include <ember/core/memory.h>
#include <ember/core/result.h>
#include <ember/core/version_types.h>

// GPU domain
#include <ember/gpu/compute.h>
#include <ember/gpu/device.h>
#include <ember/gpu/format.h>
#include <ember/gpu/frame.h>
#include <ember/gpu/frame_internal.h>
#include <ember/gpu/raster.h>
#include <ember/gpu/resources.h>
#include <ember/gpu/surface.h>
#include <ember/gpu/types.h>

// Platform domain
#include <ember/platform/filesystem.h>
#include <ember/platform/ipc.h>
#include <ember/platform/system.h>
#include <ember/platform/threading.h>
#include <ember/platform/timer.h>

// Window domain
#include <ember/window/desktop.h>
#include <ember/window/dialog.h>
#include <ember/window/events.h>
#include <ember/window/input.h>
#include <ember/window/input_codes.h>
#include <ember/window/window.h>
