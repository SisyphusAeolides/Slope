module slope_policy
  use, intrinsic :: iso_c_binding, only: c_double, c_int
  implicit none
contains
  function arach_slope_route_score(features, count) result(score) bind(C)
    real(c_double), intent(in) :: features(*)
    integer(c_int), value, intent(in) :: count
    real(c_double) :: score
    real(c_double) :: authority, liveness, boundedness, pressure

    if (count < 4_c_int) then
      score = 0.0_c_double
      return
    end if

    authority = max(0.0_c_double, min(1.0_c_double, features(1)))
    liveness = max(0.0_c_double, min(1.0_c_double, features(2)))
    boundedness = max(0.0_c_double, min(1.0_c_double, features(3)))
    pressure = max(0.0_c_double, min(1.0_c_double, features(4)))

    score = authority * 0.35_c_double + liveness * 0.25_c_double &
      + boundedness * 0.35_c_double - pressure * 0.20_c_double
    score = max(0.0_c_double, min(1.0_c_double, score))
  end function arach_slope_route_score
end module slope_policy
