#include "Reductions.h"
#include "Timer.h"

#include <algorithm>
#include <string>

float Norm(const float (&x)[XDIM][YDIM][ZDIM], int line)
{
    Timer timer;
    timer.Start();
    
    float result = 0.;

#pragma omp parallel for reduction(max:result)
    for (int i = 1; i < XDIM-1; i++)
    for (int j = 1; j < YDIM-1; j++)
    for (int k = 1; k < ZDIM-1; k++)
        result = std::max(result, std::abs(x[i][j][k]));

    timer.Stop("KERNEL Norm() on line  " + std::to_string(line) + " : Time = ");

    return result;
}

float InnerProduct(const float (&x)[XDIM][YDIM][ZDIM], const float (&y)[XDIM][YDIM][ZDIM], int line)
{
    Timer timer;
    timer.Start();
    
    double result = 0.;

#pragma omp parallel for reduction(+:result)
    for (int i = 1; i < XDIM-1; i++)
    for (int j = 1; j < YDIM-1; j++)
    for (int k = 1; k < ZDIM-1; k++)
        result += (double) x[i][j][k] * (double) y[i][j][k];

    timer.Stop("KERNEL InnerProduct() on line  " + std::to_string(line) + " : Time = ");

    return (float) result;
}
