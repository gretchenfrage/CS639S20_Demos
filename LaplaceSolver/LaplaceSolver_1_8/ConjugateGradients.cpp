#include "ConjugateGradients.h"
#include "Laplacian.h"
#include "PointwiseOps.h"
#include "Reductions.h"
#include "Substitutions.h"
#include "Utilities.h"
#include "Timer.h"

#include <iostream>

extern Timer timerLaplacian;
extern Timer timerSaxpy;

void ComputeLaplacian2(const float (&u)[XDIM][YDIM][ZDIM], float (&Lu)[XDIM][YDIM][ZDIM])
{   
#pragma omp parallel for
    for (int i = 1; i < XDIM-1; i++)
    for (int j = 1; j < YDIM-1; j++)
    for (int k = 1; k < ZDIM-1; k++)
        Lu[i][j][k] =
            -6 * u[i][j][k]
            + u[i+1][j][k]
            + u[i-1][j][k]
            + u[i][j+1][k]
            + u[i][j-1][k]
            + u[i][j][k+1]
            + u[i][j][k-1];
}

#define CHANGES true

void ComputeLaplacianImpl(
    CSRMatrix& matrix,
    const float (&u)[XDIM][YDIM][ZDIM],
    float (&Lu)[XDIM][YDIM][ZDIM])
{
    timerLaplacian.Restart();
    if (CHANGES) {
        ComputeLaplacian2(u, Lu);
    } else {
        ComputeLaplacian(matrix, u, Lu);
    }
    timerLaplacian.Pause();
}

void ConjugateGradients(
    CSRMatrix& matrix,
    CSRMatrix& L,
    float (&x)[XDIM][YDIM][ZDIM],
    const float (&f)[XDIM][YDIM][ZDIM],
    float (&p)[XDIM][YDIM][ZDIM],
    float (&r)[XDIM][YDIM][ZDIM],
    float (&z)[XDIM][YDIM][ZDIM],
    const bool writeIterations)
{
    // Algorithm : Line 2
    ComputeLaplacianImpl(matrix, x, z);
    //timerLaplacian.Restart(); ComputeLaplacian(matrix, x, z); timerLaplacian.Pause();
    Saxpy(z, f, r, -1);
    float nu = Norm(r);

    // Algorithm : Line 3
    if (nu < nuMax) return;

    // Algorithm : Line 4
    Copy(r, p);
    ForwardSubstitution(L, &p[0][0][0]);
    BackwardSubstitution(L, &p[0][0][0]);
    float rho=InnerProduct(p, r);
        
    // Beginning of loop from Line 5
    for(int k=0;;k++)
    {
        std::cout << "Residual norm (nu) after " << k << " iterations = " << nu << std::endl;

        // Algorithm : Line 6
        //timerLaplacian.Restart(); ComputeLaplacian(matrix, p, z); timerLaplacian.Pause();
        ComputeLaplacianImpl(matrix, p, z);
        float sigma=InnerProduct(p, z);

        // Algorithm : Line 7
        float alpha=rho/sigma;

        // Algorithm : Line 8
        timerSaxpy.Restart(); Saxpy(z, r, -alpha); timerSaxpy.Pause();
        nu=Norm(r);

        // Algorithm : Lines 9-12
        if (nu < nuMax || k == kMax) {
            timerSaxpy.Restart(); Saxpy(p, x, alpha); timerSaxpy.Pause();
            std::cout << "Conjugate Gradients terminated after " << k << " iterations; residual norm (nu) = " << nu << std::endl;
            if (writeIterations) WriteAsImage("x", x, k, 0, 127);
            return;
        }

        // Algorithm : Line 13
        Copy(r, z);
        ForwardSubstitution(L, &z[0][0][0]);
        BackwardSubstitution(L, &z[0][0][0]);
        float rho_new = InnerProduct(z, r);

        // Algorithm : Line 14
        float beta = rho_new/rho;

        // Algorithm : Line 15
        rho=rho_new;

        // Algorithm : Line 16
        timerSaxpy.Restart(); Saxpy(p, x, alpha); timerSaxpy.Pause();
        Saxpy(p, z, p, beta);

        if (writeIterations) WriteAsImage("x", x, k, 0, XDIM/2);
    }

}
