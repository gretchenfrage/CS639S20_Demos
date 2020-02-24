#include "PointwiseOps.h"
#include "Timer.h"

#include <string>

void Copy(const float (&x)[XDIM][YDIM][ZDIM], float (&y)[XDIM][YDIM][ZDIM], int line)
{
    Timer timer;
    timer.Start();
    
#pragma omp parallel for    
    for (int i = 1; i < XDIM-1; i++)
    for (int j = 1; j < YDIM-1; j++)
    for (int k = 1; k < ZDIM-1; k++)
        y[i][j][k] = x[i][j][k];
    
    timer.Stop("KERNEL Copy() on line  " + std::to_string(line) + " : Time = ");
}

void Saxpy(const float (&x)[XDIM][YDIM][ZDIM], const float (&y)[XDIM][YDIM][ZDIM],
    float (&z)[XDIM][YDIM][ZDIM],
    const float scale, int line, std::string nth)
{
    Timer timer;
    timer.Start();
    
    // Should we use OpenMP parallel for here?
    for (int i = 1; i < XDIM-1; i++)
    for (int j = 1; j < YDIM-1; j++)
    for (int k = 1; k < ZDIM-1; k++)
        z[i][j][k] = x[i][j][k] * scale + y[i][j][k];
    
    timer.Stop("KERNEL " + nth + "Saxpy() on line " + std::to_string(line) + " : Time = ");
}
