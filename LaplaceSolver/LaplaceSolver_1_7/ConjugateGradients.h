#pragma once

#include "Parameters.h"

void ConjugateGradients(
    const float (&x)[XDIM][YDIM][ZDIM],
    const float (&f)[XDIM][YDIM][ZDIM],
    const bool writeIterations = true);

