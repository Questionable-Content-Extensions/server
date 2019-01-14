using System;
using System.Threading.Tasks;

namespace QCExtensions.Application.Interfaces
{
	public interface ITokenValidator
	{
		Task<bool> IsValidAsync(Guid token);
	}
}