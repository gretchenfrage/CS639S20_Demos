#include "Reductions.h"
#include "changes.h"

#include <mkl.h>
#include <algorithm>

float Norm(const float (&x)[XDIM][YDIM][ZDIM])
{
    float result = 0.;

#pragma omp parallel for reduction(max:result)
    for (int i = 1; i < XDIM-1; i++)
    for (int j = 1; j < YDIM-1; j++)
    for (int k = 1; k < ZDIM-1; k++)
        result = std::max(result, std::abs(x[i][j][k]));

    return result;
}

float InnerProduct(const float (&x)[XDIM][YDIM][ZDIM], const float (&y)[XDIM][YDIM][ZDIM])
{
    if (CHANGES) {
        int n = XDIM * YDIM * ZDIM;
        const float *x_ptr = &x[0][0][0];
        const float *y_ptr = &y[0][0][0];

        float dot = cblas_sdot(n, x_ptr, 1, y_ptr, 1);
        return dot;
    } else {
        double result = 0.;

        #pragma omp parallel for reduction(+:result)
        for (int i = 1; i < XDIM-1; i++)
        for (int j = 1; j < YDIM-1; j++)
        for (int k = 1; k < ZDIM-1; k++)
            result += (double) x[i][j][k] * (double) y[i][j][k];

        return (float) result;
    }
}
