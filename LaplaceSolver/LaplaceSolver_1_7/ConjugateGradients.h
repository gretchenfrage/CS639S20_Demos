#pragma once

#include "Parameters.h"

void ConjugateGradients(
    float (&x)[XDIM][YDIM][ZDIM],
    float (&f)[XDIM][YDIM][ZDIM],
    bool writeIterations = true);