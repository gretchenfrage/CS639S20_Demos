#include "MatMatMultiply.h"
#include "Timer.h"
#include "Utilities.h"

#include <iostream>
#include <iomanip>

void MatMatMultiplyReference(const float (&A)[MATRIX_SIZE][MATRIX_SIZE],
    const float (&B)[MATRIX_SIZE][MATRIX_SIZE], float (&C)[MATRIX_SIZE][MATRIX_SIZE]);

float MatrixMaxDifference(const float (&A)[MATRIX_SIZE][MATRIX_SIZE],const float (&B)[MATRIX_SIZE][MATRIX_SIZE]);

int main(int argc, char *argv[])
{
    bool test_correctness = false;
    if (test_correctness)
    {
        float *Araw = static_cast<float*>( AlignedAllocate( MATRIX_SIZE * MATRIX_SIZE * sizeof(float), 64 ) );
        float *Braw = static_cast<float*>( AlignedAllocate( MATRIX_SIZE * MATRIX_SIZE * sizeof(float), 64 ) );
        float *Craw = static_cast<float*>( AlignedAllocate( MATRIX_SIZE * MATRIX_SIZE * sizeof(float), 64 ) );

        using matrix_t = float (&) [MATRIX_SIZE][MATRIX_SIZE];

        matrix_t A = reinterpret_cast<matrix_t>(*Araw);
        matrix_t B = reinterpret_cast<matrix_t>(*Braw);
        matrix_t C = reinterpret_cast<matrix_t>(*Craw);

        InitializeMatrices(A, B);

        Timer timer;

        for(int test = 1; test <= 10; test++)
        {
            std::cout << "Running test iteration " << std::setw(2) << test << " ";
            timer.Start();
            MatMatMultiply(A, B, C);
            timer.Stop("Elapsed time : ");
        }
        
        return 0;
    } else {
        float *Araw = static_cast<float*>( AlignedAllocate( MATRIX_SIZE * MATRIX_SIZE * sizeof(float), 64 ) );
        float *Braw = static_cast<float*>( AlignedAllocate( MATRIX_SIZE * MATRIX_SIZE * sizeof(float), 64 ) );
        float *Craw = static_cast<float*>( AlignedAllocate( MATRIX_SIZE * MATRIX_SIZE * sizeof(float), 64 ) );
        float *referenceCraw = static_cast<float*>( AlignedAllocate( MATRIX_SIZE * MATRIX_SIZE * sizeof(float), 64 ) );

        using matrix_t = float (&) [MATRIX_SIZE][MATRIX_SIZE];

        matrix_t A = reinterpret_cast<matrix_t>(*Araw);
        matrix_t B = reinterpret_cast<matrix_t>(*Braw);
        matrix_t C = reinterpret_cast<matrix_t>(*Craw);
        matrix_t referenceC = reinterpret_cast<matrix_t>(*referenceCraw);

        InitializeMatrices(A, B);
        Timer timer;

        // scramble em' boys
        InitializeMatrices(C, referenceC);

        // Correctness test
        std::cout << "Running candidate kernel for correctness test ... " << std::flush;
        timer.Start();
        MatMatMultiply(A, B, C);
        timer.Stop("Elapsed time : ");
        
        std::cout << "Running reference kernel for correctness test ... " << std::flush;
        timer.Start();
        MatMatMultiplyReference(A, B, referenceC);
        timer.Stop("Elapsed time : ");

        float discrepancy = MatrixMaxDifference(C, referenceC);
        std::cout << "Discrepancy between two methods : " << discrepancy << std::endl;
        
        for(int test = 1; test <= 20; test++)
        {
            std::cout << "Running kernel for performance run #" << std::setw(2) << test << " ... ";
            timer.Start();
            MatMatMultiply(A, B, C);
            timer.Stop("Elapsed time : ");
        }
        
        return 0;
    }
}

void MatMatMultiplyReference(const float (&A)[MATRIX_SIZE][MATRIX_SIZE],
    const float (&B)[MATRIX_SIZE][MATRIX_SIZE], float (&C)[MATRIX_SIZE][MATRIX_SIZE])
{
#pragma omp parallel for
    for (int i = 0; i < MATRIX_SIZE; i++)
    for (int j = 0; j < MATRIX_SIZE; j++) {
        C[i][j] = 0.;
        for (int k = 0; k < MATRIX_SIZE; k++)
            C[i][j] += A[i][k] * B[k][j];
    }
}

float MatrixMaxDifference(const float (&A)[MATRIX_SIZE][MATRIX_SIZE],const float (&B)[MATRIX_SIZE][MATRIX_SIZE])
{
    float result = 0.;
    for (int i = 0; i < MATRIX_SIZE; i++)
    for (int j = 0; j < MATRIX_SIZE; j++)
        result = std::max( result, std::abs( A[i][j] - B[i][j] ) );
    return result;
}
