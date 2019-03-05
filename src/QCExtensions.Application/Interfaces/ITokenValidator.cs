using System;
using System.Threading.Tasks;

namespace QCExtensions.Application.Interfaces
{
	public interface ITokenValidator
	{
		Task<bool> IsValidAsync(Guid token);
		Task<bool> HasPermissionsAsync(Guid token, Permission permissions);
	}
}